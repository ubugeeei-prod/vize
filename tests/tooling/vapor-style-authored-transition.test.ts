import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertAuthoredStylePackets,
  assertPreservedStylePacket,
  type StylePacketGroup,
} from "./support/vapor-style-authored-transition.ts";

const root = new URL(
  "../_fixtures/differential/compiler/vapor-literal-object-styles-10749/",
  import.meta.url,
);
const directory = "authored-module-successor-10701/";
const bytes = (file: string) => fs.readFileSync(new URL(file, root));
const read = (file: string) => JSON.parse(bytes(file));
const capture = read(`${directory}capture.json`);
const groups: StylePacketGroup[] = ["main", "attribute-case", "review", "priority", "sfc"];
const packet = (group: StylePacketGroup, side: "base" | "head") =>
  read(
    group === "main"
      ? side === "base"
        ? "observed-head.json"
        : `${directory}head.json`
      : `${directory}${side}-${group}.json`,
  );

test("the complete actual style successor captures retain source and archive custody", () => {
  assert.equal(capture.sourceHead, "6fc192271eafbccaac463104ca27af61b205f58f");
  assert.equal(capture.sourceTree, "afe2befecf8c9375dd4edfd3fca300608646cfec");
  assert.equal(capture.runId, 40494324373);
  assert.equal(capture.artifactId, 12729352671);
  assert.equal(
    capture.artifactDigest,
    "sha256:973096b09f9d3fb0f589c4258127a381498a44d41d1f441bba3d802950647c00",
  );
  assert.equal(Object.keys(capture.rawFiles).length, 9);
  for (const [file, digest] of Object.entries(capture.rawFiles))
    assert.equal(
      createHash("sha256")
        .update(bytes(directory + file))
        .digest("hex"),
      digest,
      file,
    );
});

test("all180 original packets retain every component, map, diagnostic and unchanged module", () => {
  const before = packet("main", "base");
  const after = packet("main", "head");
  assert.equal(before.length, 180);
  assertAuthoredStylePackets("main", after);
  const changed: number[] = [];
  after.forEach((row, index) => {
    assertPreservedStylePacket("main", index, before[index].output, row.output);
    if (row.output.module !== before[index].output.module) {
      changed.push(index);
      assert.equal(row.standalone, false);
      assert.match(row.family, /^jsx(?:-merge)?$/);
      assert.match(
        row.output.module,
        /const App = \(\{\n(?:__ssrInlineRender: true,\n)?__vapor: true,/,
      );
      assert.match(row.output.module, /setup\(\) \{/);
    }
  });
  assert.deepEqual(
    changed,
    [
      8, 10, 20, 22, 32, 34, 44, 46, 56, 58, 68, 70, 80, 82, 92, 94, 104, 106, 116, 118, 128, 130,
      140, 142, 172, 174, 176, 178,
    ],
  );
  assert.deepEqual(capture.transitions.main, { indices: changed, facets: ["module"] });
});

test("all144 CSSOM, HTML attribute case and priority packets keep complete successors", () => {
  for (const group of groups.slice(1, 4)) {
    const before = packet(group, "base");
    const after = packet(group, "head");
    assert.equal(before.length, 48);
    assertAuthoredStylePackets(group, after);
    const changed: number[] = [];
    after.forEach((row, index) => {
      if (row.fold !== true)
        assertPreservedStylePacket(group, index, before[index].output, row.output);
      if (row.output.module !== before[index].output.module) {
        changed.push(index);
        assert.equal(row.family, "jsx");
        assert.equal(row.standalone, false);
        assert.equal(before[index].output.errors.length, 1);
        assert.match(
          before[index].output.errors[0],
          /authored module preservation is not supported/,
        );
        assert.deepEqual(row.output.errors, []);
        assert.match(row.output.module, /export const App\s*=\s*\(\{\n__vapor: true,/);
      }
    });
    assert.deepEqual(
      changed,
      group === "attribute-case" ? [4, 10, 16, 22, 28, 34, 40, 46] : [0, 6, 12, 18, 24, 30, 36, 42],
    );
    assert.deepEqual(capture.transitions[group], {
      indices: changed,
      facets: ["errors", "module"],
    });
  }
});

test("released script-setup expose changes preserve the complete SFC packet", () => {
  const before = packet("sfc", "base");
  const after = packet("sfc", "head");
  assert.equal(after.length, 4);
  assertAuthoredStylePackets("sfc", after);
  for (const index of [0, 1, 3]) {
    assertPreservedStylePacket("sfc", index, before[index].output, after[index].output);
    if (index !== 0)
      assert.deepEqual(after[index].output, {
        ...before[index].output,
        code: before[index].output.code.replace(
          "setup(__props) {",
          "setup(__props, { expose: __expose }) {\n    __expose();",
        ),
      });
  }
});

test("complete packet verification rejects extra, missing and mutated outputs", () => {
  for (const group of groups) {
    const rows = packet(group, "head");
    assert.throws(() => assertAuthoredStylePackets(group, rows.slice(1)));
    assert.throws(() => assertAuthoredStylePackets(group, [...rows, rows[0]]));
    for (const facet of Object.keys(rows[0].output)) {
      const mutated = structuredClone(rows);
      mutated[0].output[facet] = "mutated output";
      assert.throws(() => assertAuthoredStylePackets(group, mutated), `${group}:${facet}`);
    }
    const index = capture.transitions[group].indices[0];
    const historical = packet(group, "base")[index].output;
    const successor = rows[index].output;
    assert.throws(() =>
      assertPreservedStylePacket(group, index, { ...historical, extra: true }, successor),
    );
    assert.throws(() =>
      assertPreservedStylePacket(group, index, historical, {
        ...successor,
        module: "wrong module",
      }),
    );
  }
});
