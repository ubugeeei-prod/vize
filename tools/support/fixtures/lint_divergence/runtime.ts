import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";
import {
  get,
  object,
  projectString,
  projectStrings,
  readJson,
  errorMessage,
  parseJsonSource,
  canonicalJson,
  text,
  type Json,
} from "./value.ts";
import type { Args } from "./selection.ts";
import type { BaselineRuleSetting } from "./compare.ts";
import { collectBaselineFindings, type LintRecord } from "./records.ts";
import {
  baselineAuthoritySha256,
  BaselineShardAccumulator,
  DEFAULT_BASELINE_SHARD_LIMITS,
  partitionBaselineFiles,
  type BaselineShardLimits,
} from "./baseline_shards.ts";

const baselineProviderPackage = "eslint-plugin-vue-scorecard";

function resolveScorecardHostRuntime(root: string) {
  const benchPackageJson = path.join(root, "tools/benchmarks/scripts/package.json"),
    requireFromBench = createRequire(benchPackageJson),
    benchManifest = readJson(benchPackageJson),
    ruleMap = readJson(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json")),
    providerManifest = requireFromBench(`${baselineProviderPackage}/package.json`) as Json,
    providerName = text(get(providerManifest, "name")),
    providerVersion = text(get(providerManifest, "version")),
    upstreamPackage = text(get(ruleMap, "upstream", "package")),
    upstreamVersion = text(get(ruleMap, "upstream", "version")),
    declared = text(get(benchManifest, "devDependencies", baselineProviderPackage));
  if (upstreamPackage !== "eslint-plugin-vue") throw new Error("lint rule-map package drifted");
  if (providerName !== upstreamPackage) throw new Error("lint baseline package name drifted");
  if (declared !== `npm:${upstreamPackage}@${providerVersion}`)
    throw new Error(`${baselineProviderPackage} must be an exact npm alias pin`);
  if (providerVersion !== upstreamVersion)
    throw new Error(`${baselineProviderPackage} version must match the lint rule map`);
  const packageVersion = (name: string): string => {
    const version = text(get(requireFromBench(`${name}/package.json`) as Json, "version"));
    if (!version) throw new Error(`${name} package must record a version`);
    return version;
  };
  return {
    plugin: requireFromBench(baselineProviderPackage) as { rules: Record<string, unknown> },
    version: providerVersion,
    runtimeVersions: {
      eslint: packageVersion("eslint"),
      "eslint-plugin-vue": providerVersion,
      "vue-eslint-parser": packageVersion("vue-eslint-parser"),
      "@typescript-eslint/parser": packageVersion("@typescript-eslint/parser"),
    },
  };
}

export interface Launch {
  command: string;
  prefix: string[];
}
export interface BaselineRawSink {
  append(results: readonly Json[]): void;
}
function exitStatus(status: number | null): number {
  const value = status ?? 1;
  return process.platform === "win32" && value >= 0x80000000 ? value - 0x100000000 : value;
}
function parseOutput(output: Uint8Array): Json {
  return parseJsonSource(new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(output));
}
export function resolveVizeLaunch(root: string, vizeBin: string | null): Launch {
  const executable = process.platform === "win32" ? "vize.exe" : "vize";
  const absolute = (filename: string) =>
    path.isAbsolute(filename) ? filename : path.join(root, filename);
  const candidates = [vizeBin, process.env.VIZE_BIN]
    .filter((value): value is string => value !== undefined && value !== null)
    .map(absolute);
  candidates.push(
    ...["ci", "debug", "release"].map((profile) => path.join(root, "target", profile, executable)),
  );
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      let resolved = candidate;
      try {
        resolved = fs.realpathSync.native(candidate);
      } catch {
        /* Match Rust's best-effort canonicalization. */
      }
      return { command: resolved, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}
export function collectRunEvidence(root: string): Json {
  const git = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    commitSha: process.env.GITHUB_SHA ?? git.stdout?.trim() ?? "",
    runtime: { name: "node", version: process.version },
    machine: {
      platform: process.platform === "win32" ? "windows" : process.platform,
      arch: process.arch,
      logicalCpuCount: os.availableParallelism(),
    },
  };
}
export function runPatina(
  cwd: string,
  project: Json,
  args: Args,
  launch: Launch,
  files: string[],
  ruleOptionsConfig: string | null = null,
) {
  const excluded =
    get(project, "lintDivergenceExcludePaths") === null
      ? []
      : projectStrings(project, "lintDivergenceExcludePaths");
  const inputs = excluded.length ? files : projectStrings(project, "vueGlobs");
  const argv = [
    ...launch.prefix,
    "lint",
    ...inputs,
    "--format",
    "json",
    ...(ruleOptionsConfig === null ? ["--no-config"] : ["--config", ruleOptionsConfig]),
    "--preset",
    args.preset ?? "all",
  ];
  const output = spawnSync(launch.command, argv, {
    cwd,
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    stdio: ["ignore", "pipe", "pipe"],
    maxBuffer: Infinity,
  });
  if (output.error) throw new Error(`vize lint failed to run: ${output.error.message}`);
  const status = exitStatus(output.status);
  if (status !== 0 && status !== 1)
    throw new Error(
      `vize lint exited with unsupported status ${status}: ${output.stderr.toString("utf8")}`,
    );
  let envelope: Json;
  try {
    envelope = parseOutput(output.stdout);
  } catch (error) {
    throw new Error(`vize lint produced invalid JSON: ${errorMessage(error)}`);
  }
  if (!Array.isArray(envelope)) throw new Error("vize lint envelope must be an array");
  if (envelope.length !== files.length)
    throw new Error(
      `${projectString(project, "id")}: vize lint reported ${envelope.length} files, expected ${files.length}`,
    );
  const findings: Json[] = [];
  for (const entry of envelope) {
    const file = get(entry, "file");
    if (typeof file !== "string") throw new Error("vize lint entry must carry file");
    const messages = get(entry, "messages");
    if (!Array.isArray(messages)) throw new Error("vize lint entry must carry messages");
    for (const message of messages) {
      // serde_json indexing upgrades null to an object and rejects other scalars.
      const value = object(message);
      if (message !== null && !value) throw new Error("vize lint message must be an object");
      findings.push({ ...value, file });
    }
  }
  return { results: envelope, findings };
}

export function eslintBaselineProgram(root: string): string {
  return path.join(root, "tools/support/fixtures/lint_divergence/eslint-baseline.ts");
}

function retainAvailableRuleMessages(
  parsed: Json,
  rules: Record<string, BaselineRuleSetting>,
  label: string,
): Json {
  const output = object(parsed),
    rawResults = get(output, "rawResults"),
    enabled = new Set(Object.keys(rules));
  if (!output) throw new Error(`${label} output must be an object`);
  if (!Array.isArray(rawResults)) throw new Error(`${label}.rawResults must be an array`);
  let droppedConfigMessageCount = 0;
  const results = rawResults.map((result, resultIndex): Json => {
    const resultObject = object(result),
      messages = get(result, "messages");
    if (!resultObject) throw new Error(`${label}.rawResults[${resultIndex}] must be an object`);
    if (!Array.isArray(messages))
      throw new Error(`${label}.rawResults[${resultIndex}].messages must be an array`);
    const retained = messages.filter((message) => {
      const ruleId = get(message, "ruleId");
      if (ruleId === null || (typeof ruleId === "string" && enabled.has(ruleId))) return true;
      droppedConfigMessageCount++;
      return false;
    });
    return retained.length === messages.length ? result : { ...resultObject, messages: retained };
  });
  return { ...output, results, droppedConfigMessageCount };
}

function hasParseError(result: Json): boolean {
  const messages = get(result, "messages");
  return Array.isArray(messages) && messages.some((message) => get(message, "ruleId") === null);
}

function logBaselineIdentity(
  authoritySha256: string,
  merged: ReturnType<BaselineShardAccumulator["finish"]>,
): void {
  if (process.env.VIZE_LINT_BASELINE_PROGRESS !== "1") return;
  console.log(
    `ESLint baseline complete: authoritySha256=${authoritySha256}, ` +
      `rawPacketCount=${merged.rawPacketCount}, rawPacketSha256=${merged.rawPacketSha256}, ` +
      `retainedPacketCount=${merged.retainedPacketCount}, ` +
      `retainedPacketSha256=${merged.retainedPacketSha256}`,
  );
}

export function runEslintBaseline(
  root: string,
  cwd: string,
  files: string[],
  rules: Record<string, BaselineRuleSetting>,
  limits: BaselineShardLimits | null = DEFAULT_BASELINE_SHARD_LIMITS,
  rawSink?: BaselineRawSink,
) {
  const runtime = resolveScorecardHostRuntime(root),
    plugin = runtime.plugin,
    availableRules = Object.fromEntries(
      Object.entries(rules).filter(([name]) =>
        Object.hasOwn(plugin.rules, name.replace(/^vue\//u, "")),
      ),
    ),
    authoritySha256 = baselineAuthoritySha256(cwd, files, availableRules as Json);
  if (!Object.keys(availableRules).length) {
    const results = files.map((file): Json => ({ filePath: path.join(cwd, file), messages: [] }));
    const accumulator = new BaselineShardAccumulator(cwd, files, {
      retainRawResults: rawSink === undefined,
      retainResults: rawSink === undefined,
    });
    accumulator.add({
      files,
      output: {
        version: runtime.version,
        runtimeVersions: {},
        rawResults: results,
        results,
        droppedConfigMessageCount: 0,
      },
    });
    const merged = accumulator.finish();
    if (baselineAuthoritySha256(cwd, files, availableRules as Json) !== authoritySha256)
      throw new Error("baseline file or rule authority changed during measurement");
    logBaselineIdentity(authoritySha256, merged);
    rawSink?.append(results);
    return {
      version: runtime.version,
      runtimeVersions: {},
      rules: availableRules,
      rawResults: rawSink ? [] : results,
      results: rawSink ? [] : results,
      droppedConfigMessageCount: 0,
      authoritySha256,
      rawPacketCount: merged.rawPacketCount,
      rawPacketSha256: merged.rawPacketSha256,
      retainedPacketCount: merged.retainedPacketCount,
      retainedPacketSha256: merged.retainedPacketSha256,
      classification: collectBaselineFindings(results, cwd),
      parseErrorResults: [],
    };
  }
  const shards =
    limits === null
      ? [{ files: [...files], sourceBytes: 0 }]
      : partitionBaselineFiles(cwd, files, limits);
  const accumulator = new BaselineShardAccumulator(cwd, files, {
      retainRawResults: rawSink === undefined,
      retainResults: rawSink === undefined,
    }),
    classification = {
      findings: [] as LintRecord[],
      parseErrorCount: 0,
      excludedNonVueCount: 0,
      invalidRangeCount: 0,
    },
    parseErrorResults: Json[] = [];
  let baselineOutputBytes = 0;
  for (let index = 0; index < shards.length; index++) {
    const shard = shards[index],
      input: Json = {
        benchPackageJson: path.join(root, "tools/benchmarks/scripts/package.json"),
        cwd,
        files: shard.files,
        rawOnly: true,
        rules: availableRules,
      },
      output = spawnSync("node", [eslintBaselineProgram(root)], {
        cwd: root,
        input: canonicalJson(input),
        maxBuffer: limits?.maxOutputBytes ?? Infinity,
      });
    if (output.error)
      throw new Error(`eslint baseline shard ${index} failed to start: ${output.error.message}`);
    if (output.status !== 0)
      throw new Error(
        `eslint baseline shard ${index} exited ${exitStatus(output.status)}:\n${output.stdout.toString("utf8")}${output.stderr.toString("utf8")}`,
      );
    let parsed: Json;
    try {
      parsed = parseOutput(output.stdout);
    } catch (error) {
      throw new Error(
        `eslint baseline shard ${index} produced invalid JSON: ${errorMessage(error)}`,
      );
    }
    const retainedOutput = retainAvailableRuleMessages(
        parsed,
        availableRules,
        `eslint baseline shard ${index}`,
      ),
      rawResults = get(retainedOutput, "rawResults"),
      results = get(retainedOutput, "results");
    if (!Array.isArray(rawResults) || !Array.isArray(results))
      throw new Error(`eslint baseline shard ${index} must carry complete results`);
    accumulator.add({
      files: shard.files,
      output: retainedOutput,
    });
    rawSink?.append(rawResults);
    if (rawSink) {
      const shardClassification = collectBaselineFindings(results, cwd);
      for (const finding of shardClassification.findings) classification.findings.push(finding);
      classification.parseErrorCount += shardClassification.parseErrorCount;
      classification.excludedNonVueCount += shardClassification.excludedNonVueCount;
      classification.invalidRangeCount += shardClassification.invalidRangeCount;
      for (const result of results) {
        if (hasParseError(result)) parseErrorResults.push(result);
      }
    }
    baselineOutputBytes += output.stdout.byteLength;
    if (process.env.VIZE_LINT_BASELINE_PROGRESS === "1") {
      const memory = process.memoryUsage();
      console.log(
        `ESLint baseline shard ${index + 1}/${shards.length}: ${shard.files.length} files, ` +
          `${output.stdout.byteLength} bytes, ${baselineOutputBytes} cumulative bytes, ` +
          `${memory.heapUsed} heap bytes, ${memory.rss} RSS bytes`,
      );
    }
  }
  const merged = accumulator.finish();
  if (baselineAuthoritySha256(cwd, files, availableRules as Json) !== authoritySha256)
    throw new Error("baseline file or rule authority changed during measurement");
  logBaselineIdentity(authoritySha256, merged);
  return {
    rawResults: merged.rawResults,
    results: merged.results,
    rules: availableRules,
    runtimeVersions: merged.runtimeVersions,
    version: merged.version,
    droppedConfigMessageCount: merged.droppedConfigMessageCount,
    authoritySha256,
    rawPacketCount: merged.rawPacketCount,
    rawPacketSha256: merged.rawPacketSha256,
    retainedPacketCount: merged.retainedPacketCount,
    retainedPacketSha256: merged.retainedPacketSha256,
    classification: rawSink ? classification : collectBaselineFindings(merged.results, cwd),
    parseErrorResults: rawSink ? parseErrorResults : merged.results.filter(hasParseError),
  };
}
