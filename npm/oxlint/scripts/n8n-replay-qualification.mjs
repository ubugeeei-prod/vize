import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  disabledFiles,
  compareBytes,
  editorPrefix,
  effectiveRules,
  fixturePath,
  layerBytes,
  manifest,
  sha256,
  verifyCorpus,
} from "./n8n-replay-inputs.mjs";
import { expectedHostCalls, qualifyN8nHost, verifyNativeCalls } from "./n8n-host-replay.mjs";

const activeNames = (rules) =>
  Object.entries(rules)
    .filter(([, entry]) => (Array.isArray(entry) ? entry[0] : entry) !== "off")
    .map(([name]) => name.slice(5))
    .sort(compareBytes);
const nativeOptions = (rules, names) => {
  const result = {};
  const fields = {
    "vue/attribute-hyphenation": "attributeHyphenation",
    "vue/component-name-in-template-casing": "componentNameInTemplateCasing",
    "vue/sfc-element-order": "sfcElementOrder",
  };
  for (const name of names) {
    const entry = rules["vize/" + name];
    if (Array.isArray(entry) && fields[name]) result[fields[name]] = entry[1];
  }
  return result;
};

export function beginN8nReplay({ root, packageDir, artifacts, receipt, binary }) {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  for (const key of [
    "VIZE_N8N_NATIVE_CUSTODY",
    "VIZE_N8N_REPLAY_OUTPUT",
    "VIZE_N8N_REPLAY_PHASE",
    "VIZE_PREFER_WORKSPACE_BINDING",
  ])
    assert.ok(!process.env[key], `ambient ${key} cannot qualify the replay`);
  const output = path.join(artifacts, "n8n");
  fs.mkdirSync(output, { recursive: true });
  const laws = spawnSync(process.execPath, ["--test", "scripts/n8n-replay-inputs.test.mjs"], {
    cwd: packageDir,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(output, "input-laws.json"), JSON.stringify(laws, null, 2) + "\n");
  assert.equal(laws.error, undefined);
  assert.equal(laws.signal, null);
  assert.equal(laws.status, 0);
  const hydrate = spawnSync(
    "git",
    ["submodule", "update", "--init", "--depth", "1", "--", fixturePath],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  fs.writeFileSync(path.join(output, "hydrate.json"), JSON.stringify(hydrate, null, 2) + "\n");
  assert.equal(hydrate.error, undefined);
  assert.equal(hydrate.signal, null);
  assert.equal(hydrate.status, 0);
  const inventory = verifyCorpus();
  fs.mkdirSync(path.join(output, "licenses"));
  for (const license of inventory.licenses)
    fs.copyFileSync(
      path.join(inventory.fixture, license.file),
      path.join(output, "licenses", license.file),
      fs.constants.COPYFILE_EXCL,
    );
  const custody = {
    schema: "vize.oxlint.n8n-source-native",
    version: 1,
    source: receipt.source,
    toolchain: receipt.toolchain,
    binary: { path: fs.realpathSync(binary), sha256: receipt.frozen.sha256 },
    calls: path.join(output, "native-calls.jsonl"),
    sources: path.join(output, "sources"),
  };
  const custodyPath = path.join(output, "custody.json");
  fs.writeFileSync(custodyPath, JSON.stringify(custody, null, 2) + "\n");
  fs.writeFileSync(path.join(output, "frozen-native-vize-layer.json"), layerBytes);
  const preload = fileURLToPath(new URL("./n8n-native-custody.cjs", import.meta.url));
  const environment = {
    NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(preload)}`.trim(),
    VIZE_N8N_NATIVE_CUSTODY: custodyPath,
    VIZE_N8N_REPLAY_OUTPUT: path.join(output, "bridge"),
    VIZE_N8N_REPLAY_PHASE: "frozenPerRule",
  };
  const result = spawnSync("vp", ["test", "run", "src/n8n-full-replay.test.ts"], {
    cwd: packageDir,
    env: { ...process.env, ...environment },
    encoding: "utf8",
    timeout: 900_000,
    maxBuffer: 32 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(output, "bridge-tests.json"), JSON.stringify(result, null, 2) + "\n");
  process.stdout.write(result.stdout ?? "");
  process.stderr.write(result.stderr ?? "");
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  return { output, custody, environment, inventory, hosts: [] };
}

export function replayN8nHost(replay, engine, version) {
  const output = path.join(replay.output, version);
  replay.hosts.push(qualifyN8nHost(engine, version, output, replay.environment));
}

export function finishN8nReplay(replay) {
  const bridgePhases = [
    "frozenPerRule",
    "frozenBatch",
    "scopedPerRule",
    "scopedSharedBatch",
    "scopedEffectiveBatch",
  ];
  const hostPhases = replay.hosts.flatMap(({ version }) =>
    ["stock", "baseline", "shared", "effective", "effective-original"].map(
      (mode) => `host:${version}:${mode}`,
    ),
  );
  const observed = verifyNativeCalls(replay.custody, [
    ...bridgePhases,
    "cacheControls",
    ...hostPhases,
  ]);
  assert.deepEqual(observed.inventory, replay.inventory);
  const totals = {};
  for (const phase of [...bridgePhases, ...hostPhases]) {
    const host = replay.hosts.find(({ version }) => phase.startsWith(`host:${version}:`));
    const mode = host ? phase.split(":")[2] : phase;
    const root = host ? host.original : replay.inventory.fixture;
    const counts = observed.counts.get(phase) ?? new Map();
    const refused = ["stock", "effective", "effective-original"].includes(mode);
    assert.equal(counts.size, refused ? 0 : replay.inventory.files.length);
    for (const { file } of replay.inventory.files) {
      const rules = mode.startsWith("frozen") ? manifest.adoption.rules : effectiveRules(file);
      const names = activeNames(rules);
      const expected = host
        ? expectedHostCalls(file, mode)
        : mode.endsWith("PerRule")
          ? names.length
          : mode === "scopedSharedBatch" && file.startsWith(editorPrefix)
            ? 2
            : 1;
      const filename = path.join(root, file);
      assert.equal(
        counts.get(filename) ?? 0,
        expected,
        `${phase}/${file}: physical native call count`,
      );
      const perFile = observed.calls.get(phase)?.get(filename) ?? [];
      const shared = ["shared", "scopedSharedBatch"].includes(mode);
      const perRule = ["baseline", "frozenPerRule", "scopedPerRule"].includes(mode);
      const selections = refused
        ? []
        : perRule
          ? names.map((name) => [name])
          : shared
            ? [
                activeNames(manifest.adoption.rules),
                ...(file.startsWith(editorPrefix) ? [["vue/attribute-hyphenation"]] : []),
              ]
            : [names];
      // Hosts may schedule the registered rules in a different callback order.
      // Compare activation multiplicity here, retaining actual call order and
      // all diagnostic arrays verbatim in the separate raw event stream.
      const activationOrder = (rows) => rows.map((row) => JSON.stringify(row)).sort(compareBytes);
      assert.deepEqual(
        activationOrder(perFile.map((event) => event.args[1].enabledRules)),
        activationOrder(selections),
        `${phase}/${file}: exact configured rule activation`,
      );
      for (const event of perFile) {
        const options = event.args[1];
        assert.equal(options.preset, "incremental");
        assert.equal(options.helpLevel, "none");
        const selected =
          shared && options.enabledRules.length === 51 ? manifest.adoption.rules : rules;
        const actual = Object.fromEntries(
          Object.entries(options).filter(([name]) =>
            ["attributeHyphenation", "componentNameInTemplateCasing", "sfcElementOrder"].includes(
              name,
            ),
          ),
        );
        assert.deepEqual(
          actual,
          nativeOptions(selected, options.enabledRules),
          `${phase}/${file}: literal native options`,
        );
      }
    }
    totals[phase] = [...counts.values()].reduce((a, b) => a + b, 0);
  }
  const controls = fs
    .readFileSync(path.join(replay.output, "bridge/cache-controls.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.equal(
    [...observed.counts.get("cacheControls").values()].reduce((sum, count) => sum + count, 0),
    controls.reduce((sum, row) => sum + row.calls, 0),
  );
  assert.deepEqual(verifyCorpus(), replay.inventory);
  const qualification = {
    source: replay.custody.source,
    toolchain: replay.custody.toolchain,
    binarySha256: replay.custody.binary.sha256,
    fixture: { revision: replay.inventory.revision, tree: replay.inventory.tree },
    adoptionRevision: manifest.adoption.revision,
    requirementsConfigSha256: manifest.adoption.configSha256,
    projectedLayerSha256: sha256(layerBytes),
    files: replay.inventory.files.length,
    packages: replay.inventory.packages,
    scriptlessFiles: manifest.corpus.scriptlessFiles,
    disabledFiles,
    totals,
    events: observed.events,
    processes: observed.processes,
    nativeCallsSha256: sha256(fs.readFileSync(replay.custody.calls)),
    hosts: replay.hosts,
    limits: [
      "frozen settings have no active-rule hint: per-rule calls remain unmet for one-call adoption",
      "optional root shared hints and native effective maps are separate supported optimizations",
      "per-file effective host batching remains unmet; both hosts reject override.settings without native calls",
      "direct SDK scriptless callbacks remain externally unavailable in both pinned hosts",
      "two n8n-local plugins and unrelated native/frontend/workspace layers not qualified",
      "installed packages, retired rules and upstream timing not qualified by this campaign",
    ],
  };
  fs.writeFileSync(
    path.join(replay.output, "qualification.json"),
    JSON.stringify(qualification, null, 2) + "\n",
  );
  return qualification;
}
