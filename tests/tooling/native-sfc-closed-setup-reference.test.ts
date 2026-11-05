import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  closedSetupReference,
  historicalSetupCode,
  type ExposurePacket,
} from "./support/script-setup-default-expose.ts";
import {
  assertDefaultRuntimeOptionPackets,
  setupComponentBracketReference,
} from "./support/setup-component-bracket-reference.ts";
import {
  type ParseModule,
  typescriptErasureReference,
} from "./support/typescript-erasure-reference.ts";

const fixture = JSON.parse(
  readFileSync(
    new URL("./fixtures/script-setup-default-expose-packets.json", import.meta.url),
    "utf8",
  ),
) as {
  packets: { source: string; before: ExposurePacket; after: ExposurePacket }[];
  emptySetupModules: {
    packets: { source: string; before: ExposurePacket; after: ExposurePacket }[];
  };
};

test("runtime matrix comparison retains semantic fields and excludes only timings", () => {
  const packet = { code: "export default {}", errors: [], warnings: [] };
  const row = {
    target: "dom",
    options: { inlineTemplate: true },
    single: [packet],
    batch: { results: [packet], timeMs: 1 },
    stats: { outputBytes: Buffer.byteLength(packet.code), timeMs: 2 },
    function: { code: "return {}" },
    functionBatch: { results: [{ code: "return {}" }], timeMs: 3 },
    functionStats: { outputBytes: 9, timeMs: 4 },
    syntax: { single: true },
  };
  const before = [row];
  const current = structuredClone(before);
  current[0].batch.timeMs = 11;
  current[0].stats.timeMs = 12;
  current[0].functionBatch.timeMs = 13;
  current[0].functionStats.timeMs = 14;
  const source = ["<script>export default {}</script>"];
  const parse: ParseModule = () => {
    throw new Error("JavaScript packets do not require a reprint parser");
  };
  assertDefaultRuntimeOptionPackets(before, current, source, parse);
  for (const mutate of [
    (changed: typeof row) => {
      changed.syntax.single = false;
    },
    (changed: typeof row) => {
      changed.function.code += "\nvoid 1;";
    },
    (changed: typeof row) => {
      changed.functionBatch.results[0].code += "\nvoid 1;";
    },
    (changed: typeof row) => {
      changed.functionStats.outputBytes += 1;
    },
    (changed: typeof row) => {
      changed.single[0].code += "\nvoid 1;";
    },
  ]) {
    const changed = structuredClone(current);
    mutate(changed[0]);
    assert.throws(() => assertDefaultRuntimeOptionPackets(before, changed, source, parse), {
      code: "ERR_ASSERTION",
    });
  }
});

test("the reviewed exposure edit preserves complete captured code, maps and descriptors", () => {
  assert.equal(fixture.packets.length, 3);
  for (const row of fixture.packets) {
    const untouched = structuredClone(row.before);
    assert.deepEqual(closedSetupReference(row.before, row.source), row.after);
    assert.deepEqual(row.before, untouched, "historical packets remain immutable");
  }
});

test("the full reference rejects unrelated code, map, descriptor and diagnostic mutations", () => {
  for (const row of fixture.packets) {
    const reference = closedSetupReference(row.before, row.source);
    for (const change of [
      (packet: ExposurePacket) => {
        packet.code += "\nvoid 1;";
      },
      (packet: ExposurePacket) => {
        if (typeof packet.map !== "string")
          throw new TypeError("captured native map must be encoded");
        packet.map = packet.map.replace('"mappings":"', '"mappings":"A');
      },
      (packet: ExposurePacket) => {
        packet.moduleShape!.defaultExportEnd += 1;
      },
      (packet: ExposurePacket) => {
        packet.errors = [{ code: "unexpected" }];
      },
      (packet: ExposurePacket) => {
        packet.macroArtifacts = [{ kind: "forged" }];
      },
    ]) {
      const changed = structuredClone(row.after);
      change(changed);
      assert.throws(() => assert.deepEqual(changed, reference), { code: "ERR_ASSERTION" });
    }
  }
});

test("ordinary sources and explicit exposure are not advanced again", () => {
  const before = fixture.packets[1].before;
  assert.deepEqual(closedSetupReference(before, "<script>export default {}</script>"), before);
  const after = fixture.packets[1].after;
  assert.deepEqual(closedSetupReference(after, fixture.packets[1].source), after);
});

test("setup component references reconstruct the complete bracket-read packet", () => {
  const before: ExposurePacket = {
    code: "function render() { return $setup.Lazy }\nexport default { render }",
    map: '{"mappings":"AAAA"}',
    moduleShape: {
      defaultExportStart: 41,
      defaultExportKeywordEnd: 55,
      defaultExportEnd: 66,
    },
    errors: [],
    warnings: [],
  };
  const preserved = structuredClone(before);
  assert.deepEqual(setupComponentBracketReference(before, "Lazy"), {
    ...before,
    code: 'function render() { return $setup["Lazy"] }\nexport default { render }',
    moduleShape: {
      defaultExportStart: 44,
      defaultExportKeywordEnd: 58,
      defaultExportEnd: 69,
    },
  });
  assert.deepEqual(before, preserved, "historical packets remain immutable");
  assert.deepEqual(setupComponentBracketReference({ code: "export default {}" }, "Lazy"), {
    code: "export default {}",
  });
  assert.throws(
    () => setupComponentBracketReference({ code: "$setup.Lazy + $setup.Lazy" }, "Lazy"),
    { code: "ERR_ASSERTION" },
  );
});

test("historical checksum receipts reverse only the exact initial exposure edit", () => {
  for (const row of fixture.packets) {
    assert.equal(historicalSetupCode(row.after.code, row.source), row.before.code);
    assert.throws(() => historicalSetupCode(row.before.code, row.source), {
      code: "ERR_ASSERTION",
    });
    const misplaced = row.after.code.replace("__expose();", "void 0;\n__expose();");
    assert.throws(() => historicalSetupCode(misplaced, row.source), { code: "ERR_ASSERTION" });
  }
});

test("Rust structured source maps retain the same complete coordinate contract", () => {
  for (const row of fixture.packets) {
    const before = { ...row.before, map: JSON.parse(row.before.map as string) };
    const after = { ...row.after, map: JSON.parse(row.after.map as string) };
    assert.deepEqual(closedSetupReference(before, row.source), after);
  }
});

test("empty setup returns preserve the complete three pinned SSR modules", () => {
  assert.equal(fixture.emptySetupModules.packets.length, 3);
  for (const row of fixture.emptySetupModules.packets) {
    assert.deepEqual(closedSetupReference(row.before, row.source), row.after);
    assert.equal(historicalSetupCode(row.after.code, row.source), row.before.code);
  }
});

const fromRoot = createRequire(import.meta.url);
const fromBabel = createRequire(fromRoot.resolve("@babel/core"));
const { parse } = fromBabel("@babel/parser") as { parse: ParseModule };

test("per-piece erasure advances spelling and byte offsets with authored anchors intact", () => {
  const map = JSON.stringify({
    version: 3,
    sources: ["Case.vue"],
    sourcesContent: ["const value: number = 1"],
    names: [],
    mappings: "AAAA",
  });
  const preserved = {
    code: 'const value=1;\nexport default { label: "雪", value: value };',
    map,
    moduleShape: {
      defaultExportStart: 15,
      defaultExportKeywordEnd: 29,
      defaultExportEnd: 61,
      hasSfcMainDefined: false,
    },
    errors: [],
    warnings: [],
  };
  const erased = {
    ...structuredClone(preserved),
    code: "const value = 1;\nexport default { label: '雪', value };",
    moduleShape: {
      ...preserved.moduleShape,
      defaultExportStart: 17,
      defaultExportKeywordEnd: 31,
      defaultExportEnd: 56,
    },
  };
  const untouched = structuredClone(preserved);
  assert.deepEqual(typescriptErasureReference(preserved, erased, parse), erased);
  assert.deepEqual(preserved, untouched);
  for (const mutate of [
    (packet: typeof erased) => {
      packet.code = packet.code.replace("= 1", "= 2");
    },
    (packet: typeof erased) => {
      packet.code += "\nvoid value;";
    },
    (packet: typeof erased) => {
      packet.map = packet.map.replace("AAAA", "AACA");
    },
    (packet: typeof erased) => {
      packet.moduleShape.defaultExportEnd += 1;
    },
    (packet: typeof erased) => {
      packet.moduleShape.hasSfcMainDefined = true;
    },
  ]) {
    const changed = structuredClone(erased);
    mutate(changed);
    assert.throws(
      () => assert.deepEqual(typescriptErasureReference(preserved, changed, parse), changed),
      { code: "ERR_ASSERTION" },
    );
  }
});

test("per-piece erasure keeps bundler effect annotations", () => {
  const preserved = { code: "export default /*@__PURE__*/ defineComponent({});" };
  const erased = { code: "export default /*@__PURE__*/defineComponent({})" };
  assert.deepEqual(typescriptErasureReference(preserved, erased, parse), erased);
  const changed = { code: "export default defineComponent({})" };
  assert.throws(() => typescriptErasureReference(preserved, changed, parse), {
    code: "ERR_ASSERTION",
  });
});
