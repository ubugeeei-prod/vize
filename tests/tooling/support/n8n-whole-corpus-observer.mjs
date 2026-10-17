// Research observer: complete execution is distinct from semantic equality.
// Usage: node n8n-whole-corpus-observer.mjs <producer source root> <output>
//        node n8n-whole-corpus-observer.mjs --require-parity <output>
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { expectedBuildIdentity, validateBuildReceipt } from "../../differential/build-receipt.ts";
import { verifyCorpus } from "../../../npm/oxlint/scripts/n8n-replay-inputs.mjs";
import { root, projection, validateProjection } from "./n8n-cli-config-inputs.mjs";
import { errorPacket, loadAuthoredCases } from "./n8n-cli-config-oracle.mjs";
import {
  assertUnchanged,
  copyOriginals,
  runCli,
  sha256,
  writeConfig,
  writeJson,
} from "./n8n-cli-config-workspace.mjs";
import {
  assertCompletePackets,
  assertQualifiedSummary,
  compareSemanticFindings,
  oracleLayers,
  recordCorpusPackets,
  semanticFindings,
} from "./n8n-whole-corpus-core.mjs";

const [producerArgument, outputArgument] = process.argv.slice(2);
assert.ok(producerArgument && outputArgument && process.argv.length === 4);
if (producerArgument === "--require-parity") {
  const summary = JSON.parse(fs.readFileSync(path.resolve(outputArgument, "summary.json"), "utf8"));
  assertQualifiedSummary(summary);
  console.log(
    "Full-corpus identity/severity/physical-range comparison passed; other #8142 frontiers remain open.",
  );
  process.exit(0);
}

const producerRoot = fs.realpathSync(path.resolve(producerArgument));
const output = path.resolve(outputArgument);
assert.ok(!fs.existsSync(output), "a previous campaign must never be overwritten");
fs.mkdirSync(output, { recursive: true });
const write = (file, value) => writeJson(path.join(output, file), value);
const git = (args, cwd) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const failures = [];
const comparisons = [];
let inventory;
const observations = { native: 0, eslint: 0, nativeFilePackets: 0, eslintFilePackets: 0 };
const observeFailure = (context, error) => {
  const packet = { context, error: errorPacket(error) };
  failures.push(packet);
  write("failures.json", failures);
};

try {
  assert.equal(
    producerRoot,
    fs.realpathSync(root),
    "producer and observer require the same checked source head",
  );
  validateProjection();
  assert.equal(git(["status", "--porcelain", "--untracked-files=no"], producerRoot), "");
  const identity = expectedBuildIdentity(producerRoot);
  const buildReceipt = JSON.parse(
    fs.readFileSync(
      path.join(producerRoot, `${identity.binaryPath}.differential-build.json`),
      "utf8",
    ),
  );
  write("producer.json", {
    buildReceipt,
    expected: identity,
    observerRevision: git(["rev-parse", "HEAD"], root),
  });
  validateBuildReceipt(buildReceipt, identity);
  const binary = path.join(producerRoot, identity.binaryPath);
  const version = spawnSync(binary, ["--version"], { encoding: "utf8" });
  write("version-process.json", {
    status: version.status,
    signal: version.signal,
    stdout: version.stdout,
    stderr: version.stderr,
    ...(version.error ? { error: errorPacket(version.error) } : {}),
  });
  assert.equal(version.status, 0);
  assert.equal(version.stdout.trim(), identity.cliVersion);

  const corpus = verifyCorpus(path.join(producerRoot, "tests/_fixtures/_git/n8n"));
  inventory = corpus;
  assert.equal(corpus.revision, projection.fixtureRevision);
  write("original-inputs.json", corpus);
  const workspace = path.join(output, "licensed-inputs");
  copyOriginals(workspace, corpus);
  const manifests = [];
  for (const packageRoot of Object.keys(corpus.packages)) {
    const file = `${packageRoot}/package.json`;
    const bytes = fs.readFileSync(path.join(corpus.fixture, file));
    const pinned = execFileSync("git", ["show", `HEAD:${file}`], { cwd: corpus.fixture });
    assert.deepEqual(bytes, pinned, `${file}: actual licensed-master package artifact`);
    fs.copyFileSync(path.join(corpus.fixture, file), path.join(workspace, file));
    manifests.push({ file, sha256: sha256(bytes), bytes: bytes.length });
  }
  write("licensed-package-manifests.json", manifests);

  // Build artifacts come from the authenticated producer checkout. The settings
  // package is explicitly owned, never the unlicensed adoption-branch package.
  const publicPackage = path.join(workspace, "node_modules/vize");
  fs.mkdirSync(publicPackage, { recursive: true });
  fs.copyFileSync(
    path.join(producerRoot, "npm/cli/package.json"),
    path.join(publicPackage, "package.json"),
  );
  fs.cpSync(path.join(producerRoot, "npm/cli/dist"), path.join(publicPackage, "dist"), {
    recursive: true,
  });
  assert.ok(fs.statSync(path.join(publicPackage, "dist/config.mjs")).isFile());
  const settings = path.join(workspace, "node_modules/@vize-acceptance/n8n-cli-settings");
  fs.mkdirSync(settings, { recursive: true });
  fs.writeFileSync(
    path.join(settings, "package.json"),
    JSON.stringify({ private: true, type: "module", exports: { ".": "./index.js" } }) + "\n",
  );
  fs.writeFileSync(
    path.join(settings, "index.js"),
    `export const settings = ${JSON.stringify({ linter: projection.linter })};\n`,
  );
  fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}\n');
  fs.mkdirSync(path.join(workspace, ".git"));
  const publicFiles = [publicPackage, settings].flatMap((directory) =>
    fs
      .globSync("**/*", { cwd: directory })
      .filter((file) => fs.statSync(path.join(directory, file)).isFile())
      .map((file) => ({
        file: path.relative(workspace, path.join(directory, file)),
        sha256: sha256(fs.readFileSync(path.join(directory, file))),
      })),
  );
  write("public-config-package.json", publicFiles);
  write("boundaries.json", {
    requirementsRevision: projection.requirements.revision,
    projectionSha256: sha256(
      fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
    ),
    executableFixtureRevision: corpus.revision,
    dependencies:
      "Actual licensed package manifests retained; project dependencies are not installed or replaced with stubs. This is not full-monorepo dependency qualification.",
    package:
      "Source-built public vize/config plus owned functional settings, not actual upstream adoption-branch shared package or installed public release.",
    remaining: [
      "independent positive/clean controls for every rule",
      "type-aware .vue script coverage",
      "eight retired Vue rules",
      "n8n-local plugins",
      "historical Oxlint batching/cache and script spans",
      "full-monorepo checker/formatter/editor lifecycle",
      "matched performance",
      "public release acceptance",
    ],
  });

  const requireProvider = createRequire(
    path.join(producerRoot, "tools/benchmarks/scripts/package.json"),
  );
  const providers = {};
  const loaded = {};
  for (const [name, pin] of Object.entries(loadAuthoredCases().oracleCapture.providers)) {
    const manifestPath = requireProvider.resolve(`${name}/package.json`);
    const entry = requireProvider.resolve(name);
    providers[name] = {
      version: requireProvider(`${name}/package.json`).version,
      publicEntrypoint: path.relative(path.dirname(manifestPath), entry).replaceAll("\\", "/"),
      publicEntrypointSha256: sha256(fs.readFileSync(entry)),
      packageManifestSha256: sha256(fs.readFileSync(manifestPath)),
    };
    write("providers.json", providers);
    assert.deepEqual(
      providers[name],
      pin,
      `${name}: exact committed independent provider identity`,
    );
    loaded[name] = requireProvider(name);
  }
  const officialBase = loaded["eslint-plugin-vue"].configs["flat/base"];
  write("official-base.json", {
    export: "flat/base",
    administrativeRules: Object.fromEntries(
      officialBase.flatMap(({ rules = {} }) => Object.entries(rules)),
    ),
    processors: officialBase.flatMap(({ processor }) =>
      processor === undefined ? [] : [processor],
    ),
  });

  for (const [packageRoot, count] of Object.entries(corpus.packages)) {
    const files = corpus.files
      .filter(({ file }) => file.startsWith(`${packageRoot}/`))
      .map(({ file }) => file.slice(packageRoot.length + 1));
    assert.equal(files.length, count);
    for (const mode of ["baseline", "scoped"]) {
      const config = writeConfig(workspace, packageRoot, mode);
      const recipe = oracleLayers(packageRoot, mode);
      assert.deepEqual(config.config, recipe.config);
      const snapshots = { native: [], eslint: [] };
      for (let repeat = 0; repeat < 2; repeat += 1) {
        const context = `${packageRoot}/${mode}/${repeat}`;
        const packetRoot = path.join("packets", packageRoot, mode, String(repeat));
        try {
          const packets = runCli({
            binary,
            workspace,
            packageRoot,
            files,
            config,
            receiptPath: path.join(output, packetRoot, "native-process.json"),
          });
          assertCompletePackets(packets, files, "native");
          assert.equal(
            sha256(fs.readFileSync(config.path)),
            config.sha256,
            "native configuration unchanged after invocation",
          );
          snapshots.native.push(packets);
          observations.native += 1;
          observations.nativeFilePackets += packets.length;
        } catch (error) {
          observeFailure(`${context}/native`, error);
        }
        try {
          const eslint = new loaded.eslint.ESLint({
            cwd: path.join(workspace, packageRoot),
            overrideConfigFile: true,
            overrideConfig: [
              ...officialBase,
              {
                files: ["**/*.vue"],
                languageOptions: {
                  parser: loaded["vue-eslint-parser"],
                  parserOptions: {
                    parser: loaded["@typescript-eslint/parser"],
                    ecmaVersion: "latest",
                    sourceType: "module",
                  },
                },
                linterOptions: { reportUnusedDisableDirectives: "off" },
              },
              ...recipe.layers,
            ],
          });
          const effective = {};
          const fileConfigs = {};
          for (const file of files) {
            const calculated = await eslint.calculateConfigForFile(file);
            const view = {
              rules: calculated.rules,
              parserOptions: calculated.languageOptions.parserOptions,
              linterOptions: calculated.linterOptions,
              settings: calculated.settings ?? {},
              processor: calculated.processor?.meta ?? null,
            };
            const digest = sha256(JSON.stringify(view));
            effective[digest] = view;
            fileConfigs[file] = digest;
          }
          write(path.join(packetRoot, "oracle-effective-configs.json"), {
            recipe,
            effective,
            fileConfigs,
          });
          const rawPackets = await eslint.lintFiles(files);
          // Save raw results before normalization, validation or comparison.
          write(path.join(packetRoot, "oracle-raw.json"), rawPackets);
          const captured = recordCorpusPackets(rawPackets, workspace);
          write(path.join(packetRoot, "oracle-recorded.json"), captured);
          assertCompletePackets(
            captured.packets,
            files.map((file) => `${packageRoot}/${file}`),
            "eslint",
          );
          snapshots.eslint.push(captured.packets);
          observations.eslint += 1;
          observations.eslintFilePackets += captured.packets.length;
        } catch (error) {
          write(path.join(packetRoot, "oracle-error.json"), errorPacket(error));
          observeFailure(`${context}/eslint`, error);
        }
        try {
          assertUnchanged(workspace, [
            ...corpus.files,
            ...corpus.licenses,
            ...manifests,
            ...publicFiles,
          ]);
        } catch (error) {
          observeFailure(`${context}/immutable-inputs`, error);
        }
      }
      for (const provider of ["native", "eslint"]) {
        try {
          assert.equal(
            snapshots[provider].length,
            2,
            `${packageRoot}/${mode}/${provider}: two complete observations`,
          );
          assert.deepEqual(
            snapshots[provider][0],
            snapshots[provider][1],
            `${packageRoot}/${mode}/${provider}: whole repeated packet stability`,
          );
        } catch (error) {
          observeFailure(`${packageRoot}/${mode}/${provider}/repeat`, error);
        }
      }
      if (snapshots.native.length === 2 && snapshots.eslint.length === 2) {
        try {
          const sourceForFile = (file) => fs.readFileSync(path.join(workspace, file), "utf8");
          const native = semanticFindings(
            snapshots.native[0],
            "native",
            packageRoot,
            sourceForFile,
          );
          const oracle = semanticFindings(
            snapshots.eslint[0],
            "eslint",
            packageRoot,
            sourceForFile,
          );
          const compared = { packageRoot, mode, ...compareSemanticFindings(native, oracle) };
          comparisons.push(compared);
          write(path.join("comparisons", packageRoot, `${mode}.json`), {
            native,
            oracle,
            ...compared,
          });
        } catch (error) {
          // A malformed coordinate must not prevent retaining later packages.
          observeFailure(`${packageRoot}/${mode}/physical-comparison`, error);
        }
      }
    }
  }
  assert.deepEqual(
    expectedBuildIdentity(producerRoot),
    identity,
    "producer executable/source identity unchanged",
  );
  assert.equal(
    git(["status", "--porcelain", "--untracked-files=no"], producerRoot),
    "",
    "all tracked producer source remains unchanged",
  );
  assertUnchanged(workspace, [...corpus.files, ...corpus.licenses, ...manifests, ...publicFiles]);
  verifyCorpus(corpus.fixture);
} catch (error) {
  observeFailure("campaign", error);
}

const canonicalDifferenceCount = comparisons.reduce(
  (sum, { nativeOnly, oracleOnly }) => sum + nativeOnly.length + oracleOnly.length,
  0,
);
const summary = {
  schema: "vize.n8n.whole-corpus-observer",
  version: 1,
  captureComplete: failures.length === 0 && comparisons.length === 18,
  expectedInventory: { originalFiles: 1369, scriptlessFiles: 19, packageRoots: 9 },
  observedInventory: {
    originalFiles: inventory?.files.length ?? 0,
    scriptlessFiles: inventory?.files.filter(({ scriptless }) => scriptless).length ?? 0,
    packageRoots: Object.keys(inventory?.packages ?? {}).length,
  },
  observations,
  modes: ["baseline", "scoped"],
  repeats: 2,
  comparisons,
  failures,
  canonicalDifferenceCount,
  qualification:
    "Complete licensed-source observations and derived identity/severity/full physical-range comparison only. Raw diagnostics, suppressed messages, fixes, suggestions, envelopes and failures are retained without filtering. This does not establish exhaustive 51-rule correctness, full-monorepo/editor/performance or public release adoption.",
};
write("summary.json", summary);
console.log(
  JSON.stringify({
    captureComplete: summary.captureComplete,
    canonicalDifferenceCount,
    failures: failures.length,
  }),
);
assert.equal(
  summary.captureComplete,
  true,
  "incomplete research capture; inspect every retained failure packet",
);
