import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  baselineConfig,
  resolveBaselineRuntime,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const directory = path.join(root, "crates/vize_patina/tests/fixtures/use-v-on-exact");
const fixturePath = path.join(directory, "cases.json");
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const requireProvider = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const pins = JSON.parse(
  fs.readFileSync(path.join(root, "tools/benchmarks/scripts/package.json")),
).devDependencies;
const providers = Object.fromEntries(
  ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map((name) => {
    const entry = requireProvider.resolve(name);
    let directory = path.dirname(entry);
    while (directory !== path.dirname(directory)) {
      const file = path.join(directory, "package.json");
      if (fs.existsSync(file)) {
        const bytes = fs.readFileSync(file);
        const metadata = JSON.parse(bytes);
        if (metadata.name === name) {
          assert.equal(metadata.version, pins[name], name);
          return [
            name,
            {
              version: metadata.version,
              publicEntrypoint: path.relative(directory, entry),
              publicEntrypointSha256: sha256(fs.readFileSync(entry)),
              packageManifestSha256: sha256(bytes),
            },
          ];
        }
      }
      directory = path.dirname(directory);
    }
    throw new Error(`Missing provider manifest: ${name}`);
  }),
);
const ruleImplementation = path.join(
  path.dirname(requireProvider.resolve("eslint-plugin-vue")),
  "rules/use-v-on-exact.js",
);
providers["eslint-plugin-vue"].ruleImplementationSha256 = sha256(
  fs.readFileSync(ruleImplementation),
);
const configuration = {
  rules: { [fixture.rule]: "warn" },
  baselineHelperSha256: sha256(
    fs.readFileSync(path.join(root, "tools/support/compat/fixtures/lint-divergence-baseline.mjs")),
  ),
};
const Vue = requireProvider("vue");
const vueDirectory = path.dirname(requireProvider.resolve("vue/package.json"));
const runtimeProvider = {
  version: Vue.version,
  globalProductionSha256: sha256(
    fs.readFileSync(path.join(vueDirectory, "dist/vue.global.prod.js")),
  ),
};
const physical = JSON.parse(fs.readFileSync(path.join(directory, "runtime-controls.json"), "utf8"));

function expected(source, packets) {
  const offset = (line, column) => {
    const lines = source.split("\n");
    return (
      lines.slice(0, line - 1).reduce((total, text) => total + text.length + 1, 0) + column - 1
    );
  };
  return packets
    .flatMap((packet) => packet.messages)
    .map((message) => {
      assert.equal(message.ruleId, fixture.rule);
      assert.equal(message.severity, 1);
      const start = offset(message.line, message.column);
      const end = offset(message.endLine, message.endColumn);
      return {
        key: source.slice(start, end),
        start: Buffer.byteLength(source.slice(0, start)),
        end: Buffer.byteLength(source.slice(0, end)),
      };
    });
}

async function capture() {
  const receipts = fs.mkdtempSync(path.join(os.tmpdir(), "vize-v-on-exact-oracle-"));
  const raw = { providers, configuration, captures: [] };
  const recorded = { providers, configuration, captures: [] };
  const runtime = resolveBaselineRuntime();
  const eslint = new runtime.ESLint({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: baselineConfig(runtime, configuration.rules),
  });
  for (const example of fixture.cases) {
    const stable = `owned/use-v-on-exact/${example.filename}`;
    const absolute = path.join(root, stable);
    const packets = await eslint.lintText(example.source, { filePath: absolute });
    // Persist every original field before classifying findings. Only the known
    // ephemeral filePath is replaced in the committed independent packet.
    const row = { id: example.id, source: example.source, packets };
    raw.captures.push(row);
    fs.appendFileSync(path.join(receipts, "raw.jsonl"), JSON.stringify(row) + "\n");
    recorded.captures.push({
      id: example.id,
      packets: packets.map((packet) =>
        packet.filePath === absolute ? { ...packet, filePath: stable } : packet,
      ),
    });
  }
  fs.writeFileSync(path.join(receipts, "raw.json"), JSON.stringify(raw, null, 2) + "\n");
  return { raw, recorded, receipts };
}

function validate({ raw }) {
  assert.equal(fixture.cases.length, 45);
  assert.equal(raw.captures.length, fixture.cases.length);
  for (const example of fixture.cases) {
    if (!example.original) continue;
    assert.equal(sha256(Buffer.from(example.source)), example.original.sha256, example.id);
    assert.equal(
      sha256(fs.readFileSync(path.join(directory, example.original.license.file))),
      example.original.license.sha256,
      example.id,
    );
  }
  for (const row of raw.captures) expected(row.source, row.packets);
}

if (process.argv.includes("--record")) {
  const observation = await capture();
  console.log(`Complete raw provider receipt: ${observation.receipts}/raw.json`);
  validate(observation);
  fixture.providers = providers;
  fixture.oracleCapture = observation.recorded;
  fixture.runtimeProvider = runtimeProvider;
  for (const row of observation.raw.captures) {
    const example = fixture.cases.find((example) => example.id === row.id);
    example.referenceExpected = expected(row.source, row.packets);
    if (!example.correctnessException) example.expected = example.referenceExpected;
  }
  fs.writeFileSync(fixturePath, JSON.stringify(fixture, null, 2) + "\n");
} else {
  await test("v-on exact controls and whole originals retain pinned independent packets", async (t) => {
    const observation = await capture();
    t.diagnostic(`Complete raw provider receipt: ${observation.receipts}/raw.json`);
    validate(observation);
    assert.deepEqual(providers, fixture.providers);
    assert.deepEqual(observation.recorded, fixture.oracleCapture);
    for (const row of observation.raw.captures)
      assert.deepEqual(
        expected(row.source, row.packets),
        fixture.cases.find((example) => example.id === row.id).referenceExpected,
        row.id,
      );
  });
}

test("trusted browser controls and actual Vue compiler preserve correct extra overlap findings", () => {
  assert.deepEqual(runtimeProvider, fixture.runtimeProvider);
  assert.equal(physical.rows.length, 4);
  for (const row of physical.rows) {
    assert.equal(row.vueVersion, runtimeProvider.version);
    assert.ok(row.events.every((event) => event.isTrusted && event.eventClass === "KeyboardEvent"));
    const calls = [];
    const rendered = Vue.compile(row.source)({ record: (handler) => calls.push(handler) }, []);
    // Replay exactly the fields captured from the trusted browser event, through
    // the official compiler's actual generated callbacks, not a copied guard.
    for (const callback of rendered.props.onKeydown) callback({ ...row.events[0] });
    assert.deepEqual(
      calls,
      row.events.map((event) => event.handler),
      `${row.kind}: exact=${row.exact}`,
    );
    assert.deepEqual(calls, row.exact ? ["specialized"] : ["base", "specialized"]);
  }
  const extras = fixture.cases.filter((example) => example.correctnessException);
  assert.equal(extras.length, 2);
  for (const example of extras) {
    assert.deepEqual(example.referenceExpected, [], example.id);
    assert.equal(example.expected.length, 1);
    const pair = physical.rows.filter(
      (row) => row.kind === example.correctnessException.runtimeControl,
    );
    assert.equal(pair.length, 2);
    assert.ok(
      pair.every((row) => row.source.includes(example.expected[0].key)),
      example.id,
    );
  }
});
