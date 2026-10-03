import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import {
  bytes,
  nativeHistoryDirectory,
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../npm/native/scripts/formatter-history-build.mjs";
import { viteConfigurationPlans, originalViteSources } from "./formatter-vite-source.mjs";
import {
  qualifyPublicCore,
  expectedPublic,
  observePublic,
  publicStrings,
  publicIdentity,
  assertMetadata,
} from "./formatter-vite-public.mjs";

import {
  completeConfig,
  snapshot,
  currentAddon,
  childFrame,
  qualifyReference,
  qualifiedConsumerPeer,
} from "./formatter-vite-observation.mjs";

export async function runPublicViteFormatter(root) {
  const nativeDir = path.join(root, "npm/native");
  const require = createRequire(path.join(root, "npm/builder/vite/package.json"));
  const receiptBytes = fs.readFileSync(nativeHistoryReceipt(nativeDir));
  const receipt = validateNativeHistoryBuild(nativeDir, JSON.parse(receiptBytes));
  const custody = currentAddon(root, receipt, require);
  const launcher = path.join(root, "npm/cli/bin/vize");
  const packed = ["npm/cli/bin/vize", "npm/cli/dist/cli.mjs", "npm/cli/dist/config.mjs"];
  const packageArtifacts = Object.fromEntries(
    packed.map((file) => [file, bytes(fs.readFileSync(path.join(root, file)))]),
  );
  assert.equal(
    fs.realpathSync(require.resolve("vize/config")),
    fs.realpathSync(path.join(root, "npm/cli/dist/config.mjs")),
  );
  assert(
    !/--(?:require|import|loader)\b/.test(process.env.NODE_OPTIONS ?? ""),
    "competing Node instrumentation",
  );
  const { defineConfig } = await import("../../npm/builder/vite/src/vite-plus.ts");
  const { resolveConfigExport } = await import("../../npm/builder/vite/src/config.ts");
  const { runNative, nativeBinary } =
    await import("../../npm/builder/vite/src/vite-plus/runner.ts");
  const { relocateTaskConfig } =
    await import("../../npm/builder/vite/src/vite-plus/config-paths.ts");
  const { taskConfigKey } = await import("../../npm/builder/vite/src/vite-plus/types.ts");
  assert.equal(fs.realpathSync(nativeBinary()), fs.realpathSync(launcher));
  const rootRequire = createRequire(path.join(root, "package.json"));
  const reference = rootRequire("oxfmt");
  const referencePackageBytes = fs.readFileSync(rootRequire.resolve("oxfmt/package.json"));
  const referencePackage = JSON.parse(referencePackageBytes);
  const lock = qualifyReference(root, rootRequire, referencePackage.version);
  const preload = path.join(root, "tests/differential/formatter-vite-loader.cjs");
  const plans = viteConfigurationPlans(root);
  const env = { command: "fmt", mode: "production" };
  const report = {
    schema: "vize.public-vite-formatter-result",
    version: 2,
    buildReceipt: receipt,
    preparation: custody.preparation,
    originalViteSources,
    packageArtifacts,
    preload: bytes(fs.readFileSync(preload)),
    reference: {
      name: referencePackage.name,
      version: referencePackage.version,
      package: bytes(referencePackageBytes),
    },
    runtime: {
      node: process.version,
      nodeExecutable: process.execPath,
      nodeOptions: process.env.NODE_OPTIONS ?? null,
    },
    rows: [],
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  };
  const reportPath = path.join(nativeHistoryDirectory(nativeDir), "public-vite-report.json");
  const persist = () => fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
  const originalCwd = process.cwd();
  // The ignored consumer lives under this checkout so its actual installed
  // Vite+ peer is resolved normally from package ancestors, without installs.
  const workspaceParent = path.join(
    root,
    "npm/builder/vite/target/differential/vite-config-workspaces",
  );
  fs.mkdirSync(workspaceParent, { recursive: true });
  for (const fixture of plans) {
    const workspace = fs.mkdtempSync(path.join(workspaceParent, "vize-public-vite space-"));
    const loadCapture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-vite-load-"));
    const row = { id: fixture.id, workspace, state: "running", passes: [], cleanup: null };
    report.rows.push(row);
    try {
      process.chdir(workspace);
      row.consumerPeer = qualifiedConsumerPeer(root, workspace, require, rootRequire, lock);
      report.publicCore ??= qualifyPublicCore(row.consumerPeer, lock);
      fs.writeFileSync("UserCard.vue", fixture.input);
      row.initialFiles = snapshot(workspace);
      const source = structuredClone(fixture.source);
      if (fixture.base)
        source.extends = [
          defineConfig(structuredClone(fixture.base)),
          Promise.resolve(structuredClone(fixture.promisedBase)),
        ];
      row.originalInput = {
        source: fixture.source,
        base: fixture.base ?? null,
        promisedBase: fixture.promisedBase ?? null,
        integration: fixture.integration,
      };
      const vp = await defineConfig(
        source,
        fixture.integration,
      )({ command: "build", mode: "production" });
      const originalPublic = observePublic(vp);
      row.publicConfig = originalPublic.snapshot;
      row.publicComparison = {
        actual: publicStrings(row.publicConfig),
        expected: expectedPublic(fixture),
      };
      persist();
      const metadata = vp[taskConfigKey];
      assert.deepEqual(row.publicComparison.actual, row.publicComparison.expected);
      assert.deepEqual(
        Reflect.ownKeys(vp).filter((key) => typeof key === "symbol"),
        [taskConfigKey],
      );
      assertMetadata(metadata, fixture, row);
      const resolved = await resolveConfigExport(metadata.config, env);
      row.independentResolution = {
        env,
        config: completeConfig(resolved),
        expected: completeConfig(fixture.native),
      };
      row.publicAfterResolution = publicIdentity(vp, originalPublic);
      assert.deepEqual(resolved, fixture.native);
      const transported = JSON.stringify(relocateTaskConfig(resolved, workspace));
      const oracle = await reference.format("control.ts", fixture.script, fixture.formatter);
      row.referenceResult = oracle;
      assert.deepEqual(oracle.errors, []);
      const expected = `<script setup lang="ts">\n${oracle.code}</script>\n`;
      row.expectedFiles = { "UserCard.vue": bytes(Buffer.from(expected)) };
      const changed = expected !== fixture.input;
      for (const [index, flag] of ["--check", "--write", "--check"].entries()) {
        const pass = {
          index,
          inputFiles: snapshot(workspace),
          outputFiles: null,
          process: null,
          cleanup: null,
        };
        row.passes.push(pass);
        let tempFile;
        try {
          pass.returnedStatus = await runNative(
            "fmt",
            [flag, "UserCard.vue"],
            metadata,
            undefined,
            async (command, argv) => {
              tempFile = argv[3];
              pass.request = {
                command,
                argv,
                configPath: tempFile,
                configBefore: bytes(fs.readFileSync(tempFile)),
              };
              assert.equal(command, process.execPath);
              assert.deepEqual(argv, [launcher, "fmt", "--config", tempFile, flag, "UserCard.vue"]);
              assert.equal(path.dirname(tempFile), os.tmpdir());
              assert.deepEqual(pass.request.configBefore, bytes(Buffer.from(transported)));
              const destination = path.join(loadCapture, `${index}.json`);
              const childEnv = {
                ...process.env,
                VIZE_VITE_CLI_LOAD_CAPTURE: destination,
                NODE_OPTIONS:
                  `${process.env.NODE_OPTIONS ?? ""} --require ${JSON.stringify(preload)}`.trim(),
              };
              const frame = {
                command,
                argv,
                cwd: workspace,
                processError: null,
                environment: {
                  forwardingPolicy: "inherited with declared load observer",
                  nodeOptions: childEnv.NODE_OPTIONS,
                  loadCapture: childEnv.VIZE_VITE_CLI_LOAD_CAPTURE,
                  nativeLibraryPath: childEnv.NAPI_RS_NATIVE_LIBRARY_PATH ?? null,
                  forceWasi: childEnv.NAPI_RS_FORCE_WASI ?? null,
                },
                timedOut: false,
                exitStatus: null,
                signal: null,
                stdout: null,
                stderr: null,
              };
              pass.process = frame;
              persist();
              const status = await childFrame(command, argv, childEnv, workspace, frame);
              persist();
              pass.observationErrors = [];
              for (const [label, observe] of [
                [
                  "temporary-config",
                  () => {
                    pass.request.configAfter = bytes(fs.readFileSync(tempFile));
                  },
                ],
                [
                  "actual-load",
                  () => {
                    const raw = fs.readFileSync(destination);
                    pass.loadCapture = bytes(raw);
                    pass.loads = JSON.parse(raw);
                  },
                ],
                [
                  "workspace",
                  () => {
                    pass.outputFiles = snapshot(workspace);
                  },
                ],
              ]) {
                try {
                  observe();
                } catch (error) {
                  pass.observationErrors.push({ label, name: error.name, message: error.message });
                }
                persist();
              }
              assert.deepEqual(pass.observationErrors, []);
              currentAddon(root, receipt, require);
              const nativeLoads = pass.loads.loads.filter(
                (load) => load.realpath === fs.realpathSync(custody.local),
              );
              assert.equal(
                nativeLoads.length,
                1,
                "CLI did not load exactly one actual prepared addon",
              );
              const [load] = nativeLoads;
              assert.equal(load.completed, true);
              assert.equal(load.error, null);
              assert.deepEqual(load.observationErrors, []);
              assert.equal(load.beforeSha256, receipt.frozen.sha256);
              assert.equal(load.afterSha256, receipt.frozen.sha256);
              assert(
                !pass.loads.loads.some(
                  (other) =>
                    other !== load && /vize-vitrine|@vizejs[\\/]native/.test(other.filename),
                ),
                "unexpected native addon load",
              );
              assert(
                load.exportDescriptors.some(
                  ({ key, valueType }) => key === "runCli" && valueType === "function",
                ),
              );
              assert.deepEqual(pass.request.configAfter, pass.request.configBefore);
              return status;
            },
          );
        } finally {
          pass.cleanup = {
            configPath: tempFile ?? null,
            deleted: tempFile ? !fs.existsSync(tempFile) : null,
          };
          persist();
        }
        assert.equal(pass.cleanup.deleted, true);
        assert.equal(pass.process.processError, null);
        assert.equal(pass.process.timedOut, false);
        assert.equal(pass.process.signal, null);
        assert.equal(pass.returnedStatus, index === 0 && changed ? 1 : 0);
        assert.equal(pass.process.exitStatus, pass.returnedStatus);
        assert.deepEqual(pass.process.stdout, bytes(Buffer.alloc(0)));
        const stream =
          index === 1
            ? changed
              ? "enabled-write-stderr.txt"
              : "disabled-write-stderr.txt"
            : index === 0 && changed
              ? "enabled-check-stderr.txt"
              : "enabled-recheck-stderr.txt";
        assert.deepEqual(
          pass.process.stderr,
          bytes(
            fs.readFileSync(
              path.join(
                root,
                "tests/_fixtures/differential/formatter-history/import-sorting-config",
                stream,
              ),
            ),
          ),
        );
        assert.deepEqual(pass.outputFiles, index === 0 ? row.initialFiles : row.expectedFiles);
        pass.publicAfterCall = publicIdentity(vp, originalPublic);
        persist();
      }
      row.state = "matched-reference";
    } catch (error) {
      row.state = "failed";
      row.error = { name: error.name, message: error.message, stack: error.stack };
    } finally {
      process.chdir(originalCwd);
      persist();
      try {
        fs.rmSync(workspace, { recursive: true, force: true });
        fs.rmSync(loadCapture, { recursive: true, force: true });
        row.cleanup = { removed: true };
      } catch (error) {
        row.cleanup = { removed: false, error: error.message };
        row.state = "failed";
      }
      persist();
    }
  }
  assert.equal(report.rows.length, 9);
  assert.deepEqual(
    report.rows.map(({ id }) => id),
    plans.map(({ id }) => id),
  );
  assert.equal(
    report.rows.filter(({ state }) => state === "failed").length,
    0,
    `Failed plans: ${report.rows
      .filter(({ state }) => state === "failed")
      .map(({ id }) => id)
      .join(", ")}; complete public/comparison/process packet: ${reportPath}`,
  );
  assert.equal(
    report.rows.reduce((count, row) => count + row.passes.length, 0),
    27,
  );
  return report;
}
