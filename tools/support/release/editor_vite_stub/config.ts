import assert from "node:assert/strict";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
  type JsonRecord,
} from "../../json-value.ts";
import { Recorder, success } from "../../config/windows_consumer/common.ts";
import { inventory, writeFixture } from "./filesystem.ts";
import { hook, observed, verifyJournal } from "./custody.ts";

export function configAcceptance(
  root: string,
  pnpm: string[],
  identity: JsonRecord,
  original: JsonRecord,
  record: Recorder,
): void {
  const modules = join(root, "node_modules"),
    stub = join(modules, "vite");
  const files = asRecord(original.files);
  function phase(label: string, negative = false) {
    const before = inventory(modules);
    record.json(label + ".before.json", before);
    const plain = record.run(
      label + "-original-cli",
      [...pnpm, "exec", "vize", "lint", "src/app.vue"],
      root,
    );
    const cliCustody = observed(root, identity, record, label + "-cli");
    const cli = record.run(
      label + "-json-cli",
      [
        process.execPath,
        "--require",
        hook,
        asString(identity.cli),
        "lint",
        "src/app.vue",
        "--format",
        "json",
      ],
      root,
      cliCustody.env,
    );
    const apiCustody = observed(root, identity, record, label + "-api");
    const output = join(record.root, label + ".api.json");
    const api = record.run(
      label + "-api",
      [
        process.execPath,
        "--require",
        hook,
        fileURLToPath(new URL("./config-probe.ts", import.meta.url)),
        root,
        output,
      ],
      root,
      apiCustody.env,
    );
    const value = readJsonRecord(output);
    if (negative) {
      for (const packet of [plain, cli, api]) {
        assert.equal(packet.status, packet === api ? 1 : 2, packet.label);
        assert.equal(packet.error, null);
        assert.equal(packet.signal, null);
        assert.ok((packet.stdout + packet.stderr).includes("runtime-owner-11145"), packet.label);
      }
      assert.equal(asRecord(value.error).name, "Error");
      assert.equal(asRecord(value.error).message, "runtime-owner-11145");
    } else {
      for (const packet of [plain, cli, api]) success(packet);
      const report = asArray(parseJson(cli.stdout));
      for (const item of report) {
        const row = asRecord(item);
        assert.equal(row.errorCount, 0);
        assert.equal(row.warningCount, 0);
        assert.deepEqual(row.messages, []);
      }
      assert.notEqual(value.value, null);
      assert.ok(Object.hasOwn(value, "value"));
      assert.equal(
        asRecord(asRecord(value.value).typeChecker).jsxTypecheck,
        true,
        "the original project export must produce its actual project defaults",
      );
    }
    // Rust's lint config loader exits(2) directly, bypassing Node's exit event.
    // The actual process status is checked above; its successful .node load remains required.
    verifyJournal(cliCustody.journal, identity, negative ? null : 0);
    // A runtime-owner import error precedes the JS API's native normalizer.
    // Keep that honest journal; do not preload a binding to fabricate a native call.
    verifyJournal(apiCustody.journal, identity, negative ? 1 : 0, !negative);
    const after = inventory(modules);
    record.json(label + ".after.json", after);
    assert.deepEqual(after, before, label + " changed an external installed byte, mode, or link");
    for (const name of ["vite.config.ts", "src/app.vue"])
      assert.equal(readFileSync(join(root, name), "utf8"), files[name]);
    return { config: value, report: negative ? null : parseJson(cli.stdout) };
  }
  const baseline = phase("baseline");
  mkdirSync(stub);
  writeFixture(root, files, (name) => name.startsWith("node_modules/"));
  const withStub = phase("types-only");
  assert.deepEqual(
    withStub,
    baseline,
    "full CLI report and imported config must survive the exact original stub",
  );
  writeFileSync(
    join(stub, "package.json"),
    '{"name":"vite","type":"module","main":"./index.mjs"}\n',
  );
  writeFileSync(
    join(stub, "index.mjs"),
    'export function loadConfigFromFile(){throw new Error("runtime-owner-11145")}\n',
  );
  phase("runtime-owner", true);
  rmSync(join(stub, "index.mjs"));
  writeFixture(root, files, (name) => name.startsWith("node_modules/"));
  assert.deepEqual(phase("stub-repaired"), baseline);
  rmSync(stub, { recursive: true });
  assert.deepEqual(phase("removed-repaired"), baseline);
  record.json("config-acceptance.json", {
    originalCommand: [...pnpm, "exec", "vize", "lint", "src/app.vue"],
    originalGlobExpansion: "src/*.vue expands to the single original src/app.vue",
    phases: ["baseline", "types-only", "runtime-owner", "stub-repaired", "removed-repaired"],
    baseline,
    assertion:
      "complete positive CLI/config objects equal; runtime authority error is fatal; every full installation vector unchanged within phase",
  });
}
