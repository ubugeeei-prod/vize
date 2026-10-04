import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  bytes,
  hash,
  nativeHistoryBuildEnvironment,
} from "../../npm/native/scripts/formatter-history-build.mjs";
import {
  loadPublicNativeManifest,
  NATIVE_FORMATTER_MANIFEST,
} from "../differential/formatter-native-manifest.mjs";
import { validatePublicNativeReport } from "../differential/formatter-native.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const loaded = loadPublicNativeManifest(root);
function isolated(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "public-native-contract-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const manifest = structuredClone(loaded.manifest);
  const copied = new Set();
  function copy(value) {
    if (!value || typeof value !== "object") return;
    if (typeof value.path === "string" && !copied.has(value.path)) {
      const destination = path.join(directory, value.path);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.copyFileSync(path.join(root, value.path), destination);
      copied.add(value.path);
    }
    for (const child of Object.values(value)) copy(child);
  }
  copy(manifest);
  copy(JSON.parse(fs.readFileSync(path.join(root, manifest.apiManifest.path))));
  const save = () => {
    const destination = path.join(directory, NATIVE_FORMATTER_MANIFEST);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, JSON.stringify(manifest));
  };
  save();
  return { directory, manifest, save };
}
function controlReport() {
  // Synthetic admission vectors only: no module, build or NAPI execution credit.
  const source = { head: "a".repeat(40), tree: "b".repeat(40), workingDiff: "c".repeat(64) };
  const build = {
    source,
    frozen: { sha256: "d".repeat(64) },
    toolchain: { node: process.version },
  };
  const buildRaw = Buffer.from(JSON.stringify(build));
  const observer = Buffer.from("synthetic observer source, never executed");
  const preparation = { ...source, sha256: build.frozen.sha256 };
  const runtime = {
    nodeExecutable: "/synthetic/node",
    observer: "/synthetic/observer.mjs",
    nodeOptions: null,
    cargoEnvironment: nativeHistoryBuildEnvironment(),
  };
  const rows = loaded.cases.map((fixture) => {
    let source = fixture.input.toString();
    const passes = [];
    for (let index = 0; index < (fixture.outcome === "success" ? 3 : 1); index++) {
      const input = { source, options: structuredClone(fixture.options) };
      const raw = Buffer.from(JSON.stringify(input));
      const result =
        fixture.outcome === "success"
          ? { code: fixture.expected.toString(), changed: source !== fixture.expected.toString() }
          : null;
      const packet = {
        schema: "vize.public-native-formatter-observation",
        version: 1,
        input,
        inputSha256: hash(raw),
        preparation,
        buildReceiptSha256: hash(buildRaw),
        artifactSha256: build.frozen.sha256,
        observerSha256: hash(observer),
        runtime: {
          node: process.version,
          nodeOptions: null,
          cargoEnvironment: runtime.cargoEnvironment,
          errorStackTraceLimit: 0,
        },
        result,
        error: fixture.expectedError ? structuredClone(fixture.expectedError) : null,
      };
      passes.push({
        argv: [runtime.nodeExecutable, runtime.observer],
        stdin: bytes(raw),
        stdout: bytes(Buffer.from(`${JSON.stringify(packet)}\n`)),
        stderr: bytes(Buffer.alloc(0)),
        exitStatus: 0,
        signal: null,
        processError: null,
        packet,
      });
      if (result) source = result.code;
    }
    return {
      id: fixture.id,
      outcome: fixture.outcome,
      state: "matched-reference",
      native: "unsupported",
      passes,
    };
  });
  return {
    report: {
      schema: "vize.public-native-formatter-result",
      version: 1,
      manifestSha256: loaded.manifestSha256,
      buildReceipt: build,
      preparation,
      runtime,
      rows,
      summary: {
        plannedCases: 9,
        legacyByteMatches: 8,
        legacyErrorMatches: 1,
        legacyFailures: 0,
        nativeUnsupported: 9,
        nativeHandled: 0,
        nativeEquivalent: 0,
        pairedComparisons: 0,
      },
    },
    buildRaw,
    observer,
  };
}
function accept({ report, buildRaw, observer }) {
  return validatePublicNativeReport(loaded, report, buildRaw, observer);
}
function rewritePacket(pass) {
  pass.stdout = bytes(Buffer.from(`${JSON.stringify(pass.packet)}\n`));
}

void test("public native history binds all original controls and distinct full caller options", () => {
  assert.equal(loaded.cases.length, 9);
  assert.equal(loaded.cases.filter((row) => row.controlIndex !== undefined).length, 5);
  assert.deepEqual(loaded.cases.find(({ id }) => id.endsWith("/false")).options, {
    sortImports: false,
  });
  assert.deepEqual(loaded.cases.find(({ id }) => id.endsWith("/omitted")).options, {});
  const vector = controlReport();
  accept(vector);
  assert.equal(
    vector.report.rows.reduce((count, row) => count + row.passes.length, 0),
    25,
  );
});

void test("native source admission rejects omitted authority and coherent sibling/options substitutions", (t) => {
  for (const change of [
    ({ manifest }) => {
      delete manifest.napiWitness.functions;
    },
    ({ manifest, directory }) => {
      const file = path.join(directory, manifest.napiWitness.path);
      fs.appendFileSync(file, "\n// unregistered current owner\n");
    },
    ({ manifest, directory }) => {
      manifest.napiWitness.sourceSha256 = hash(
        fs.readFileSync(path.join(directory, manifest.napiWitness.path)),
      );
    },
    ({ manifest }) => {
      manifest.cases[3] = { ...manifest.cases[4], id: manifest.cases[3].id };
    },
    ({ manifest }) => {
      manifest.cases[1].options = manifest.cases[2].options;
    },
    ({ manifest, directory }) => {
      const file = path.join(directory, manifest.viteWitness.path);
      const raw = fs
        .readFileSync(file)
        .toString()
        .replace('import B from "./b";', 'import WRONG from "./wrong";');
      assert.notEqual(raw, fs.readFileSync(file).toString());
      fs.writeFileSync(file, raw);
      manifest.viteWitness.sourceSha256 = hash(Buffer.from(raw));
    },
  ]) {
    const fixture = isolated(t);
    change(fixture);
    fixture.save();
    assert.throws(() => loadPublicNativeManifest(fixture.directory));
  }
});

void test("complete native vectors reject missing calls, changed full results/errors and forged custody", () => {
  for (const change of [
    ({ report }) => report.rows[0].passes.pop(),
    ({ report }) => {
      const pass = report.rows[0].passes[1];
      pass.packet.result.code = "partial";
      rewritePacket(pass);
    },
    ({ report }) => {
      const pass = report.rows.at(-1).passes[0];
      delete pass.packet.error.ownProperties.stack;
      rewritePacket(pass);
    },
    ({ report }) => {
      const pass = report.rows[0].passes[0];
      pass.packet.artifactSha256 = "f".repeat(64);
      rewritePacket(pass);
    },
    ({ report }) => {
      report.rows[0].passes[0].stderr = bytes(Buffer.from("unexpected warning\n"));
    },
    ({ report }) => {
      report.rows[0].passes[0].stdout.sha256 = "0".repeat(64);
    },
  ]) {
    const vector = controlReport();
    change(vector);
    assert.throws(() => accept(vector));
  }
});

void test("terminal parse failure preserves every complete earlier and failing raw process frame", () => {
  const vector = controlReport();
  const row = vector.report.rows[0];
  row.state = "failed";
  row.error = "genuine terminal raw packet is not JSON";
  row.passes[2].stdout = bytes(Buffer.from("complete invalid packet\0\n"));
  row.passes[2].packet = null;
  vector.report.summary.legacyByteMatches--;
  vector.report.summary.legacyFailures++;
  accept(vector);
  assert.equal(row.passes.length, 3);
  assert(
    Buffer.from(row.passes[2].stdout.base64, "base64").equals(
      Buffer.from("complete invalid packet\0\n"),
    ),
  );
  const forged = structuredClone(vector);
  forged.report.rows[0].passes[2].stdout.sha256 = "0".repeat(64);
  assert.throws(() => accept(forged));
});
