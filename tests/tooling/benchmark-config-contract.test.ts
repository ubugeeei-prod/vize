import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { pathToFileURL } from "node:url";
import { projectViteConfigSource } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { originalConfigDocument } from "./support/docs-config-archive.ts";
import {
  assertBenchmarkContractBinary,
  qualifyBenchmarkConfig,
  validateBenchmarkConfigContract,
  writeBenchmarkConfig,
  CONFIG_PROBE_INPUT,
  CONFIG_PROBE_OUTPUT,
  RETIRED_FORMATTER_ERROR,
} from "./support/benchmark-config-contract.ts";

/** Deliberately synthetic old/current CLI contracts; never native performance evidence. */
function binaryFixture(t: TestContext, behavior: string) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-config-law-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const binary = path.join(directory, "vize");
  fs.writeFileSync(
    binary,
    `#!${process.execPath}
const fs = require("node:fs");
const {pathToFileURL} = require("node:url");
const mode = ${JSON.stringify(behavior)};
const filename = process.argv[4];
const legacy = filename === "vize.config.json";
const output = ${JSON.stringify(CONFIG_PROBE_OUTPUT)};
(async () => {
  if (mode === "changed-binary") fs.appendFileSync(__filename, "\\n");
  if (legacy && mode !== "old" && mode !== "both" && mode !== "wrong-legacy" && mode !== "changed-binary") {
    if (mode === "extra-stdout") process.stdout.write("unexpected\\n");
    if (mode === "modified-input") fs.writeFileSync("Probe.vue", output);
    process.stderr.write(mode === "unknown" ? "permission denied\\n" : mode === "partial-retirement" ? "prefix " + ${JSON.stringify(RETIRED_FORMATTER_ERROR)} : ${JSON.stringify(RETIRED_FORMATTER_ERROR)});
    process.exitCode = 2;
    return;
  }
  let policy;
  if (legacy) policy = JSON.parse(fs.readFileSync(filename, "utf8"));
  else policy = (await import(pathToFileURL(require("node:path").resolve(filename)).href)).default.vue;
  const supports = legacy || (mode !== "old" && mode !== "ignored-canonical");
  if (mode === "wrong-legacy" && legacy) fs.writeFileSync("Probe.vue", "wrong\\n");
  else fs.writeFileSync("Probe.vue", supports && policy.formatter.tabWidth === 3 ? output : "<template>\\n  <p>hello</p>\\n</template>\\n");
})().catch(error => { process.stderr.write(String(error)); process.exitCode = 2; });
`,
    { mode: 0o755 },
  );
  return { directory, binary };
}

test("the full independent published three-space formatter recipe defines the contract probe", () => {
  const archived = originalConfigDocument("docs/content/guide/formatter-indent-width.md");
  const code = [...archived.matchAll(/^```[^\n]*\n([\s\S]*?)^```/gm)].map((match) => match[1]);
  assert.deepEqual(JSON.parse(code[0]), { formatter: { tabWidth: 3 } });
  assert.equal(code[1], CONFIG_PROBE_OUTPUT);
  assert.ok(archived.includes(`\`${CONFIG_PROBE_INPUT}\``));
});

test("actual child-process policy qualification preserves old carriers and accepts new ones only after exact retirement", async (t) => {
  for (const mode of ["old", "both", "current"]) {
    const fixture = binaryFixture(t, mode);
    const contract = qualifyBenchmarkConfig(fixture.binary);
    assert.equal(contract.mode, mode === "current" ? "canonical-vue" : "legacy-native-json");
    assertBenchmarkContractBinary(contract, fixture.binary);
    assert.equal(contract.probes.length, 2);
    assert.equal(
      contract.binary.sha256,
      createHash("sha256").update(fs.readFileSync(fixture.binary)).digest("hex"),
    );
    const workspace = path.join(fixture.directory, "workspace");
    fs.mkdirSync(workspace);
    const original = '{ "languageServer": { "typecheck":true,"editor":true } }\n';
    const evidence = writeBenchmarkConfig(workspace, original, contract);
    assert.equal(Buffer.from(evidence.original.base64, "base64").toString(), original);
    if (mode === "current") {
      assert.equal(fs.existsSync(path.join(workspace, "vize.config.json")), false);
      assert.equal(
        fs.readFileSync(path.join(workspace, "vize.config.json.fixture"), "utf8"),
        original,
      );
      const { default: config } = await import(
        pathToFileURL(path.join(workspace, "vite.config.mjs")).href
      );
      assert.deepEqual(await projectViteConfigSource(config), {
        languageServer: { typecheck: true, editor: true },
        typeChecker: { jsxTypecheck: false },
      });
    } else {
      assert.deepEqual(fs.readdirSync(workspace), ["vize.config.json"]);
      assert.equal(fs.readFileSync(path.join(workspace, "vize.config.json"), "utf8"), original);
      assert.deepEqual(evidence.current, evidence.original);
    }
  }
});

test("unknown failures, partial retirement, changed input, ignored config and executable swaps fail closed", (t) => {
  for (const mode of [
    "unknown",
    "partial-retirement",
    "extra-stdout",
    "modified-input",
    "ignored-canonical",
    "wrong-legacy",
    "changed-binary",
  ]) {
    const fixture = binaryFixture(t, mode);
    const retained = path.join(fixture.directory, "qualification.json");
    assert.throws(() => qualifyBenchmarkConfig(fixture.binary, retained), undefined, mode);
    if (mode !== "changed-binary") {
      const refused = JSON.parse(fs.readFileSync(retained, "utf8"));
      assert.equal(
        refused.probes.length,
        2,
        "failed contract selection retains both actual process observations",
      );
    }
  }
  const fixture = binaryFixture(t, "current");
  const contract = qualifyBenchmarkConfig(fixture.binary);
  const other = path.join(fixture.directory, "other");
  fs.copyFileSync(fixture.binary, other);
  assert.throws(() => assertBenchmarkContractBinary(contract, other));
});

test("retained proof rejects forged modes, whole outputs, policy bytes, process results and missing probes", (t) => {
  const fixture = binaryFixture(t, "current");
  const original = qualifyBenchmarkConfig(fixture.binary);
  for (const change of [
    (copy: typeof original) => {
      copy.mode = "legacy-native-json";
    },
    (copy: typeof original) => {
      copy.probes.pop();
    },
    (copy: typeof original) => {
      copy.probes[1].output.base64 += "AA==";
    },
    (copy: typeof original) => {
      copy.probes[1].config.sha256 = "0".repeat(64);
    },
    (copy: typeof original) => {
      copy.probes[0].status = 0;
    },
    (copy: typeof original) => {
      copy.probes[0].signal = "SIGTERM";
    },
    (copy: typeof original) => {
      copy.probes[1].error = "spawn failed";
    },
    (copy: typeof original) => {
      copy.probes[0].argv[2] = "different.json";
    },
  ]) {
    const copy = structuredClone(original);
    change(copy);
    assert.throws(() => validateBenchmarkConfigContract(copy));
  }
  const old = qualifyBenchmarkConfig(binaryFixture(t, "both").binary);
  old.mode = "canonical-vue";
  assert.throws(() => validateBenchmarkConfigContract(old));
});
