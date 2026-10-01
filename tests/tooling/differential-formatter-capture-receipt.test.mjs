import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadFormatterApiManifest } from "../differential/formatter-api.mjs";
import { sha256 } from "../differential/manifest.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/differential/formatter-history";

void test("compact receipts retain all actual capture phases and reject forged process/hash/probe chains", (t) => {
  const count = ["capture", "capture-extra", "capture-final"].map(
    (name) =>
      loadFormatterApiManifest(path.join(root, directory, `${name}-manifest.json`), root).cases
        .length,
  );
  assert.deepEqual(count, [51, 25, 4]);
  fs.mkdirSync(path.join(root, "target"), { recursive: true });
  const temporary = fs.mkdtempSync(path.join(root, "target/formatter-compact-contract-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  function write(name, value) {
    const file = path.join(temporary, name);
    const bytes = Buffer.from(JSON.stringify(value));
    fs.writeFileSync(file, bytes);
    return { path: path.relative(root, file), sha256: sha256(bytes) };
  }
  for (const mutate of [
    ({ receipt }) => {
      receipt.rows[0].inputSha256 = "0".repeat(64);
    },
    ({ receipt }) => {
      receipt.rows[0].outputSha256 = "0".repeat(64);
    },
    ({ receipt }) => {
      receipt.rows[0].repeat.count = 1;
    },
    ({ receipt }) => {
      receipt.rows[0].fixedPoint.count = 1;
    },
    ({ receipt }) => {
      receipt.rows.find((row) => !row.fixedPoint).fixedPoint = receipt.rows[0].fixedPoint;
    },
    ({ receipt }) => {
      receipt.rows.find((row) => row.initial.stdout === "empty").repeat.process = "success";
    },
    ({ tables }) => {
      tables.processes.success.signal = "SIGTERM";
    },
    ({ tables }) => {
      tables.processes.success.processError = "spawn failed";
    },
    ({ tables }) => {
      tables.probes.default.argv = ["--defaults"];
    },
    ({ tables }) => {
      tables.arguments[Object.keys(tables.arguments)[0]] = ["--json"];
    },
    ({ build }) => {
      build.exitStatus = 1;
    },
    ({ build }) => {
      build.source.observerSourceSha256 = "0".repeat(64);
    },
    ({ manifest }) => {
      manifest.cases[0].witness.sourceRef = "missing-source";
    },
    ({ manifest }) => {
      manifest.cases[0].witness.sourceSha256 = "0".repeat(64);
    },
  ]) {
    const manifest = JSON.parse(
      fs.readFileSync(path.join(root, directory, "capture-manifest.json"), "utf8"),
    );
    const receipt = JSON.parse(
      fs.readFileSync(path.join(root, manifest.captureReceipt.path), "utf8"),
    );
    const tables = JSON.parse(fs.readFileSync(path.join(root, receipt.tables.path), "utf8"));
    const build = JSON.parse(fs.readFileSync(path.join(root, tables.build.path), "utf8"));
    mutate({ manifest, receipt, tables, build });
    tables.build = write("build.json", build);
    receipt.tables = write("tables.json", tables);
    manifest.captureReceipt = write("receipt.json", receipt);
    const file = path.join(temporary, "manifest.json");
    fs.writeFileSync(file, JSON.stringify(manifest));
    assert.throws(() => loadFormatterApiManifest(file, root));
  }
});
