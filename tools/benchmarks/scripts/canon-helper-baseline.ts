/** First actual wired helper baseline. All runs are explicitly untimed observers. */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadAdmissionAPI } from "./canon-helper-admission-api.ts";
import {
  prepareHelperArchitectureCarrier,
  repairHelperArchitectureCarrier,
} from "./canon-template-helper-carrier.ts";
import {
  prepareHelperProvider,
  sourceMapperSdk,
  observerLauncher,
} from "./canon-helper-baseline-provider.ts";
import { assertHelperObservation } from "./canon-helper-baseline-observe.ts";
import { helperCensus } from "./canon-helper-baseline-census.ts";
import { hash, identify, run, save } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue } from "./canon-instance-directives-model.ts";

async function main() {
  const [providerArg, binaryArg, regenerationArg, driverArg, outputArg, countArg] =
    process.argv.slice(2);
  assert.ok(providerArg && binaryArg && regenerationArg && driverArg && outputArg && countArg);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    regeneration = path.resolve(regenerationArg);
  const driver = path.resolve(driverArg),
    output = path.resolve(outputArg),
    count = Number(countArg);
  assert.ok([1, 10, 100, 2500].includes(count));
  fs.mkdirSync(output, { recursive: false });
  const { checker, receipt: providerReceipt } = await prepareHelperProvider(
    provider,
    binary,
    regeneration,
    output,
  );
  const { api, identities } = await loadAdmissionAPI(driver);
  const version = run(output, "source-version", binary, ["--version"], output);
  assert.equal(version.status, 0);
  assert.match(version.stdout.toString("utf8"), /0\.474\.0-dev/u);
  const providerFiles = [
    "package-lock.json",
    ...["vize", "vue", "typescript", "typescript-native"].map(
      (name) => "node_modules/" + name + "/package.json",
    ),
  ];
  const protectedBefore = providerFiles.map((file) => ({
    file,
    ...identify(path.join(provider, file)),
  }));
  const entries: unknown[] = [],
    failures: { label: string; error: string }[] = [];
  save(path.join(output, "context.json"), {
    timingUsable: false,
    normalTiming: null,
    rustAllocator: null,
    sourceRootAttested: false,
    invocationRole:
      "normal image baseline and separate instrumentation, no before/after speed claim",
    cpu: {
      model: os.cpus()[0]?.model ?? null,
      logicalCpus: os.cpus().length,
      affinity: fs.existsSync("/proc/self/status")
        ? fs.readFileSync("/proc/self/status", "utf8")
        : null,
    },
    generator: { revision: "b31168ec9c5282f4b58de1d9f5f64e8aa7b7bdaf", identities },
    provider: providerReceipt,
    environment: Object.fromEntries(
      ["GOMAXPROCS", "RAYON_NUM_THREADS", "GOGC", "GOMEMLIMIT", "GODEBUG", "GOEXPERIMENT"].map(
        (key) => [key, process.env[key] ?? null],
      ),
    ),
  });
  for (const stage of ["valid", "invalid", "poisoned"] as const) {
    const root = path.join(output, stage);
    fs.mkdirSync(root);
    const project = path.join(root, "project");
    const corpus = prepareHelperArchitectureCarrier(
      project,
      provider,
      {
        sfcs: count,
        sfcBytes: 2048,
        graphNodes: 8,
        shape: "diamond",
        typeWidth: 4,
        typeDepth: 4,
        stage,
        allowLarge: true,
      },
      api,
    );
    const sdk = sourceMapperSdk(project, provider, [binary, "content-mapper"]);
    save(path.join(root, "source-sdk-override.json"), sdk);
    const captureInput = (name: string) => {
      for (const entry of corpus.manifest) {
        const destination = path.join(root, name, "inputs", entry.file);
        fs.mkdirSync(path.dirname(destination), { recursive: true });
        fs.copyFileSync(path.join(project, entry.file), destination);
      }
    };
    for (const repaired of stage === "valid" ? [false] : [false, true]) {
      const expected = repaired
        ? repairHelperArchitectureCarrier(project, corpus).expected
        : corpus.expected;
      const phase = repaired ? "repaired" : "original";
      captureInput(phase);
      for (const instrumentation of ["normal-image", "native-observer"] as const) {
        for (const tool of ["cli", "mapper"] as const) {
          const label = [stage, phase, instrumentation, tool].join("/");
          const out = path.join(root, phase, instrumentation, tool);
          fs.mkdirSync(out, { recursive: true });
          const env = {
            ...process.env,
            XDG_CACHE_HOME: path.join(out, "cache"),
            TMPDIR: path.join(out, "tmp"),
          };
          fs.mkdirSync(env.TMPDIR);
          const stats = instrumentation === "native-observer";
          const native = stats
            ? observerLauncher(path.join(out, "native"), checker, "stats")
            : checker;
          const rpc = stats ? observerLauncher(path.join(out, "rpc"), binary, "rpc") : null;
          const manifestFile = path.join(project, "node_modules/vize/package.json");
          const manifest = jsonValue(fs.readFileSync(manifestFile, "utf8"));
          assert.ok(
            isRecord(manifest) &&
              isRecord(manifest.typescript) &&
              isRecord(manifest.typescript.contentMapper),
          );
          manifest.typescript.contentMapper.exec = rpc ? [rpc] : [binary, "content-mapper"];
          save(manifestFile, manifest);
          const args =
            tool === "cli"
              ? [
                  "check",
                  "--no-config",
                  "--format",
                  "json",
                  "--show-virtual-ts",
                  "--tsconfig",
                  "tsconfig.json",
                  "--corsa-path",
                  native,
                ]
              : [
                  "--runExternalCode",
                  "--checkers",
                  "1",
                  "--noEmit",
                  "--pretty",
                  "false",
                  "--project",
                  "tsconfig.mapper.json",
                ];
          const command = tool === "cli" ? binary : native;
          try {
            const observed = run(out, "program", command, args, project, env);
            const cli = assertHelperObservation(
              tool,
              observed.stdout,
              observed.status,
              project,
              expected,
              out,
            );
            const virtualFiles =
              cli?.result.files.filter((file) => file.file.endsWith(".vue")) ?? [];
            if (cli)
              assert.equal(
                virtualFiles.length,
                count,
                "every authored SFC must return its virtual source",
              );
            const census = cli ? await helperCensus(provider, virtualFiles, out) : null;
            const entry = {
              label,
              passed: true,
              expected: expected.length,
              stdout: hash(observed.stdout),
              stderr: hash(observed.stderr),
              census: census?.total ?? null,
              timingUsable: false,
            };
            entries.push(entry);
            save(path.join(out, "acceptance.json"), entry);
          } catch (error) {
            const value = {
              label,
              error: error instanceof Error ? (error.stack ?? error.message) : String(error),
            };
            failures.push(value);
            save(path.join(out, "failure.json"), value);
          }
        }
      }
    }
  }
  for (const entry of protectedBefore)
    assert.deepEqual(identify(path.join(provider, entry.file)), {
      path: entry.path,
      bytes: entry.bytes,
      sha256: entry.sha256,
    });
  save(path.join(output, "acceptance.json"), {
    sfcs: count,
    entries,
    failures,
    passed: failures.length === 0,
    timingUsable: false,
    accuracyScope:
      "all authored mutation identities, multiplicities and repairs; not full type soundness or a latency result",
    statisticsScope:
      "native extended statistics and forced GC are untimed observers; normal-image commands have no statistics flags",
  });
  assert.equal(failures.length, 0, "raw failures remain in the evidence packet");
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
