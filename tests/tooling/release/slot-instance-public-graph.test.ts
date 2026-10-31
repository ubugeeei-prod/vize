import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { asArray, asRecord, asString, readJsonRecord } from "../../../tools/support/json-value.ts";
import { Recorder } from "../../../tools/support/config/windows_consumer/common.ts";
import {
  captureDeclarationGraph,
  sha256,
  type DeclarationGraph,
} from "../../../tools/support/release/computed_event_custody.ts";
import {
  assertGraph,
  expected,
  profiles,
  type Role,
} from "../../../tools/support/release/slot_instance_public/contract.ts";
import { installedGraph } from "../../../tools/support/release/slot_instance_public/installed-graph.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const corpus =
  "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/";
// These whole source records predate the edge expectation and contain actual
// declaration bodies, not edges synthesized from the expected graph.
const sources = {
  before: {
    path: corpus + "slot-provider-payload-update-11027.json",
    sha256: "0a71c5c21535e50d02df527e6c00267dfd8b44bfc0d63c37c49a55d67ab1166c",
  },
  head: {
    path: corpus + "public-slot-instance-update-11023.json",
    sha256: "3f815c1abf70c0a93792e1b5c4c8fadef270cfbb3e8ebddd9f0d0a8280c25616",
  },
};
function materialize(role: Role, directory: string) {
  const raw = readFileSync(join(root, sources[role].path));
  assert.equal(sha256(raw), sources[role].sha256);
  const source = asRecord(JSON.parse(raw.toString("utf8")));
  const files = asArray(source.files).map((value) => asRecord(value));
  const sdk = asArray(expected(role, "esm").sdk).map((value) => asRecord(value));
  assert.equal(sdk.length, profiles[role].declarations);
  for (const file of sdk) {
    const name = asString(file.file);
    const original = files.find((entry) => entry.path === "npm/cli/" + name);
    assert(original);
    const bytes = Buffer.from(
      asString(role === "before" ? original.afterWholeUtf8 : original.wholeUtf8),
    );
    assert.equal(bytes.length, file.bytes);
    assert.equal(sha256(bytes), file.sha256);
    mkdirSync(dirname(join(directory, name)), { recursive: true });
    writeFileSync(join(directory, name), bytes, { flag: "wx" });
  }
}

test("authenticated nine/ten whole source bodies produce independent package-relative graphs on real files", () => {
  const work = realpathSync(mkdtempSync(join(tmpdir(), "slot-graph-real-")));
  try {
    for (const role of ["before", "head"] as const) {
      const directory = join(work, role),
        record = new Recorder(join(work, role + "-evidence"));
      materialize(role, directory);
      const graph = installedGraph(directory, role, record);
      assert.equal(graph.files.length, profiles[role].declarations);
      assert.equal(graph.edges.length, profiles[role].edges);
      assert.deepEqual(
        asRecord(readJsonRecord(join(record.root, "installed-declaration-graph.json"))).graph,
        graph,
      );
      assertGraph(role, directory, captureDeclarationGraph(directory));
      assert.throws(() => assertGraph(role === "head" ? "before" : "head", directory, graph));
      assert.throws(() => assertGraph(role, join(work, "wrong-root"), graph));
      const sameLengthWrong = directory.slice(0, -1) + "x";
      assert.throws(() => assertGraph(role, sameLengthWrong, graph));
      for (const mutation of [
        (value: DeclarationGraph) => value.files.pop(),
        (value: DeclarationGraph) => value.files.push(value.files[0]!),
        (value: DeclarationGraph) => {
          value.files[0]!.sha256 = "0".repeat(64);
        },
        (value: DeclarationGraph) => {
          value.files[0]!.path = join(work, "foreign.d.ts");
        },
        (value: DeclarationGraph) => value.edges.reverse(),
        (value: DeclarationGraph) => {
          value.edges[0]!.specifier += ".extra";
        },
        (value: DeclarationGraph) => {
          value.edges[0]!.attributes = {};
        },
        (value: DeclarationGraph) => {
          value.edges[0]!.kind = "import";
        },
        (value: DeclarationGraph) => {
          value.edges[0]!.target = join(directory, "foreign.d.ts");
        },
        (value: DeclarationGraph) => {
          value.edges[0]!.target = join(work, "foreign.d.ts");
        },
        (value: DeclarationGraph) => {
          value.edges[0]!.from = join(work, "foreign.d.ts");
        },
      ]) {
        const changed = structuredClone(graph);
        mutation(changed);
        assert.throws(() => assertGraph(role, directory, changed));
      }
    }
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});

test("a real body mutation fails while complete graph and declaration bytes survive before assertion", () => {
  const work = realpathSync(mkdtempSync(join(tmpdir(), "slot-graph-failure-")));
  try {
    for (const role of ["before", "head"] as const) {
      const directory = join(work, role),
        record = new Recorder(join(work, role + "-evidence"));
      materialize(role, directory);
      const file = join(directory, "virtual-ts.d.ts");
      const changed = Buffer.concat([
        readFileSync(file),
        Buffer.from("\n// changed declaration body\n"),
      ]);
      writeFileSync(file, changed);
      assert.throws(() => installedGraph(directory, role, record));
      const capture = readJsonRecord(join(record.root, "installed-declaration-graph.json"));
      const graph = asRecord(capture.graph);
      assert.equal(capture.root, directory);
      assert.equal(asArray(graph.files).length, profiles[role].declarations);
      assert.equal(asArray(graph.edges).length, profiles[role].edges);
      assert.deepEqual(readFileSync(join(record.root, "sdk--virtual-ts.d.ts")), changed);
      assert.equal(
        readJsonRecord(join(record.root, "installation-phase.json")).phase,
        "validate-declaration-graph",
      );
    }
    const historical = readFileSync(
      join(root, "tools/support/release/slot_instance_public/declaration-edges.json.txt"),
    );
    assert.equal(
      sha256(historical),
      "22f282468418c4f0a98aacc332e7beaee7309db5bd29c0383a63a088c1c2aa8b",
    );
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
});
