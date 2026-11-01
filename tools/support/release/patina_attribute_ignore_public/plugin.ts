import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdirSync, readFileSync, readdirSync, realpathSync, writeFileSync } from "node:fs";
import { join, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
  type JsonRecord,
} from "../../json-value.ts";
import { hook, verifyJournal } from "../editor_vite_stub/custody.ts";
import { preserve, type Context } from "./bootstrap.ts";
import { errorPacket } from "./native.ts";
import {
  corpus,
  custody,
  digest,
  expectedEslint,
  presentation,
  referenceFor,
  role,
  rule,
  sourceFor,
  type AttributeCase,
} from "./contract.ts";

const observerModule = fileURLToPath(import.meta.url);
export const observerOption = "--import=" + pathToFileURL(observerModule).href;
export function assertObserverOptions(options: unknown): void {
  assert.equal(options, observerOption, "only the exact owned observer preload is admitted");
}
/** Observe each actual Node worker's original loader, without replacing native functions. */
export function observeWorker(): void {
  assertObserverOptions(process.env.NODE_OPTIONS);
  const descriptorPath = asString(process.env.VIZE_ATTR_PUBLIC_OBSERVER);
  assert.equal(realpathSync(descriptorPath), descriptorPath);
  const raw = readFileSync(descriptorPath);
  assert(raw.length > 0 && raw.length <= 16384);
  const descriptor = asRecord(parseJson(raw.toString()));
  assert.equal(descriptor.schema, "vize.attribute-ignore.worker-observer.v1");
  assert.equal(descriptor.role, role);
  assert.equal(descriptor.preload, observerModule);
  assert.equal(descriptor.hook, hook);
  custody();
  const directory = asString(descriptor.directory),
    root = asString(descriptor.installRoot);
  assert.equal(realpathSync(directory), directory);
  assert.equal(realpathSync(root), root);
  assert(descriptorPath.startsWith(directory + sep));
  assert(!directory.startsWith(root + sep));
  const identity = asRecord(descriptor.identity),
    binary = asRecord(identity.binary);
  if (process.env.VIZE_PUBLIC_NATIVE_CUSTODY) {
    const inherited = asRecord(parseJson(process.env.VIZE_PUBLIC_NATIVE_CUSTODY));
    assert.equal(inherited.installRoot, root);
    assert.equal(inherited.nativePath, binary.path);
    assert.equal(inherited.nativeSha256, binary.sha256);
    assert(asString(inherited.journalPath).startsWith(directory + sep));
  }
  const configuration = {
    schema: "vize-public-native-custody-v1",
    installRoot: root,
    nativePath: binary.path,
    nativeSha256: binary.sha256,
    journalPath: join(directory, asString(descriptor.label) + "." + process.pid + ".ndjson"),
  };
  const owned = process.env.NODE_OPTIONS;
  delete process.env.NODE_OPTIONS;
  process.env.VIZE_PUBLIC_NATIVE_CUSTODY = JSON.stringify(configuration);
  try {
    createRequire(import.meta.url)(hook);
  } finally {
    process.env.NODE_OPTIONS = owned;
  }
}

interface Host {
  verify(source: string, config: unknown[], options: { filename: string }): unknown[];
  verifyAndFix(source: string, config: unknown[], options: { filename: string }): unknown;
}
export async function runEslint(context: Context): Promise<void> {
  const require = createRequire(join(context.root, "package.json"));
  const original = context.native.lintPatinaSfc;
  const Linter = asRecord(require("eslint")).Linter as new () => Host;
  const parser: unknown = require("vue-eslint-parser");
  const entry: unknown = await import(pathToFileURL(context.eslint).href);
  const plugin = asRecord(entry).default,
    host = new Linter();
  const packets: unknown[] = [];
  for (const row of corpus.valid) {
    const source = sourceFor(row),
      reference = referenceFor(row);
    const config = [
      {
        files: ["**/*.vue"],
        languageOptions: { parser },
        plugins: { vize: plugin },
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        rules: { ["vize/" + rule]: ["warn", row.policy] },
      },
    ];
    const options = { filename: row.filename };
    const partial: JsonRecord = { id: row.id, source, policy: row.policy, filename: row.filename };
    packets.push(partial);
    const save = () => context.record.json("eslint-packets.json", { complete: false, packets });
    const observed = <T>(name: string, invoke: () => T): T => {
      partial.pending = name;
      save();
      try {
        const value = invoke();
        partial[name] = value;
        delete partial.pending;
        save();
        return value;
      } catch (error) {
        partial[name + "Error"] = errorPacket(error);
        save();
        throw error;
      }
    };
    const verify = observed("verify", () => host.verify(source, config, options));
    const fixResult = observed("fixResult", () => host.verifyAndFix(source, config, options));
    const output = asString(asRecord(fixResult).output);
    const requery = observed("requery", () => host.verify(output, config, options));
    const secondFixResult = observed("secondFixResult", () =>
      host.verifyAndFix(output, config, options),
    );
    const packet = {
      id: row.id,
      source,
      policy: row.policy,
      verify,
      fixResult,
      requery,
      secondFixResult,
    };
    Object.assign(partial, packet);
    context.record.json("eslint-packets.json", { complete: false, packets });
    assert.deepEqual(verify, expectedEslint(row), row.id);
    assert.deepEqual(
      fixResult,
      { fixed: row.expectedCount > 0, messages: [], output: reference.fixedSource },
      row.id,
    );
    assert.deepEqual(requery, [], row.id);
    assert.deepEqual(
      secondFixResult,
      { fixed: false, messages: [], output: reference.fixedSource },
      row.id,
    );
    assert.equal(context.native.lintPatinaSfc, original);
    assert.equal(asRecord(require.cache[context.binary]?.exports).lintPatinaSfc, original);
    preserve(context);
  }
  context.record.json("eslint-packets.json", {
    complete: true,
    inputs: 9,
    packets,
    loadedNative: { path: context.binary, originalMethodIdentityPreserved: true },
    limits:
      "Original nine whole public ESLint packets; separate target eslint10.13.0/vue-eslint-parser10.4.1 and frozen reference eslint-plugin-vue10.9.2/eslint10.4.1/vue-eslint-parser10.4.1/@typescript-eslint/parser8.65.0 identities.",
  });
}

export function strictHostJson(stdout: string): JsonRecord {
  return asRecord(parseJson(stdout));
}
function verifyWorkers(context: Context, directory: string, label: string): unknown[] {
  const names = readdirSync(directory).filter(
    (name) => name.startsWith(label + ".") && name.endsWith(".ndjson"),
  );
  assert(names.length > 0, "the real host must emit worker-loader journals");
  const records = [];
  let loaded = 0;
  for (const name of names) {
    const path = join(directory, name),
      raw = readFileSync(path, "utf8");
    assert(raw.endsWith("\n"));
    const events = raw
      .slice(0, -1)
      .split("\n")
      .map((line) => asRecord(parseJson(line)));
    const last = events.at(-1)!;
    const exit = last.event === "exit" ? asNumber(last.code) : null;
    verifyJournal(path, context.identity, exit, false);
    const count = events.filter(
      (event) => event.event === "returned" && event.expectedNative === true,
    ).length;
    loaded += count;
    records.push({
      path,
      sha256: digest(raw),
      events,
      loadedNative: count,
      exitObserved: last.event === "exit",
    });
  }
  assert(loaded > 0, "the actual host workers must load the authenticated public Vize image");
  return records;
}
export function runOxlint(context: Context): void {
  const directory = join(context.proof, "oxlint-native");
  mkdirSync(directory);
  const configPath = join(context.root, ".oxlintrc.attribute.json");
  let calls = 0;
  const observe = (row: AttributeCase, options: unknown[]): JsonRecord[] => {
    const source = sourceFor(row),
      input = join(context.root, row.filename);
    const config = {
      plugins: ["vue"],
      jsPlugins: [context.plugin],
      settings: { vize: { helpLevel: "none", preset: "incremental" } },
      rules: { "no-unused-vars": "off", ["vize/" + rule]: ["warn", ...options] },
    };
    const configBytes = JSON.stringify(config, null, 2) + "\n";
    writeFileSync(input, source);
    writeFileSync(configPath, configBytes);
    const label = "oxlint-" + calls++,
      descriptor = join(directory, label + ".observer.json");
    context.record.json(label + ".input.json", { id: row.id, source, options, config });
    writeFileSync(
      descriptor,
      JSON.stringify({
        schema: "vize.attribute-ignore.worker-observer.v1",
        role,
        directory,
        installRoot: realpathSync(context.root),
        identity: context.identity,
        preload: observerModule,
        hook,
        label,
      }) + "\n",
      { flag: "wx" },
    );
    const env = {
      ...process.env,
      NODE_OPTIONS: observerOption,
      VIZE_ATTR_PUBLIC_OBSERVER: descriptor,
    };
    const packet = context.record.run(
      label,
      [process.execPath, context.oxlint, "--config", configPath, "--format", "json", row.filename],
      context.root,
      env,
    );
    context.record.json(label + ".raw-observation.json", {
      id: row.id,
      source,
      options,
      config,
      packet,
      after: readFileSync(input, "utf8"),
      observer: { NODE_OPTIONS: observerOption, VIZE_ATTR_PUBLIC_OBSERVER: descriptor },
    });
    assert.equal(packet.error, null);
    assert.equal(packet.signal, null);
    assert.equal(packet.status, 0, row.id);
    assert.equal(readFileSync(input, "utf8"), source);
    assert.equal(readFileSync(configPath, "utf8"), configBytes);
    const payload = strictHostJson(packet.stdout),
      diagnostics = asArray(payload.diagnostics).map((value) => asRecord(value));
    assert.deepEqual(
      diagnostics.map((finding) => ({
        code: finding.code,
        message: finding.message,
        severity: finding.severity,
        filename: finding.filename,
      })),
      referenceFor(row).verify[0]!.messages.map((finding) => ({
        code: "vize(" + rule + ")",
        severity: "warning",
        filename: row.filename,
        message: `${presentation(finding).message} (at <template>:${finding.line}:${finding.column})`,
      })),
      row.id,
    );
    const workers = verifyWorkers(context, directory, label);
    context.record.json(label + ".checked-observation.json", {
      id: row.id,
      source,
      options,
      config,
      packet,
      payload,
      workers,
    });
    preserve(context);
    return diagnostics;
  };
  const baselines = new Map<string, { row: AttributeCase; findings: JsonRecord[] }>();
  for (const id of ["always-default", "never-default"]) {
    const row = corpus.valid.find((value) => value.id === id)!;
    baselines.set(asString(row.policy), { row, findings: observe(row, [row.policy]) });
  }
  for (const row of corpus.valid) {
    const style = typeof row.policy === "string" ? row.policy : (row.policy.style ?? "always");
    const baseline = baselines.get(style)!,
      original = referenceFor(baseline.row).verify[0]!.messages;
    const retained = referenceFor(row).verify[0]!.messages.map((finding) => {
      const index = original.findIndex(
        (candidate) =>
          candidate.line === finding.line &&
          candidate.column === finding.column &&
          candidate.endLine === finding.endLine &&
          candidate.endColumn === finding.endColumn &&
          candidate.messageId === finding.messageId,
      );
      assert(index >= 0);
      return baseline.findings[index];
    });
    const forms =
      typeof row.policy === "string"
        ? [[row.policy]]
        : [[row.policy], [style, { ignore: row.policy.ignore ?? [] }]];
    for (const options of forms) assert.deepEqual(observe(row, options), retained, row.id);
  }
  assert.equal(calls, 18);
  context.record.json("oxlint-complete.json", {
    complete: true,
    inputs: 9,
    calls,
    limits:
      "Independent semantic vector plus whole retained same-host diagnostic presentation. All original raw stdout/stderr and native-loader journals are retained. No direct-host precise-range/fixer or unobserved worker-exit qualification.",
  });
}

if (process.env.VIZE_ATTR_PUBLIC_OBSERVER) observeWorker();
