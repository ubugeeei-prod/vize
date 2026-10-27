import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  canonicalPaths,
  comparePackets,
  readAssets,
  relocatePacket,
  type Row,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const assets = readAssets(root, root);
const bytes = (text: string) => Buffer.byteLength(text);

function fixture() {
  const source = "<template>日本 🧭</template>";
  // An authored comment with the same spelling must never become generated metadata.
  const prefix = "  // @vize-map: 0:0 -> 0:0\n" + "// padded\n".repeat(120);
  const body = "const value = '日本 🧭';\n";
  const tail = "const result = value;\n";
  const packet = (helper: Buffer) => {
    const g0 = bytes(prefix) + helper.length,
      g1 = g0 + bytes(body);
    const comment = `  // @vize-map: ${g0}:${g1} -> 0:${bytes(source)}\n`;
    const target = g1 + bytes(comment);
    return {
      text: prefix + helper.toString() + body + comment + tail,
      extension: ".tsx",
      mappings: [
        [0, bytes(prefix) - 1, 0, 1, 1, 0],
        [g0, bytes(body), 0, bytes(source), 1, 1048575],
        [target, bytes(tail), 1, 1, 1, 0],
      ],
      semanticLinks: [
        {
          sourceStart: g0,
          sourceLength: 5,
          targetStart: target,
          targetLength: 5,
          kind: "original-kind",
        },
      ],
      diagnosticDirectives: {
        unusedExpectDirectiveDiagnostics: [],
        directives: [[0, 1, g0, g1, 0, 0]],
      },
      diagnostics: [{ start: 0, length: 1, code: 100001, messageText: "original message" }],
    };
  };
  return { source, base: packet(assets.embedded), head: packet(assets.imported) };
}

test("certified helper and self-described coordinates preserve complete packets", () => {
  const { source, base, head } = fixture();
  const proof = relocatePacket(base, head, source, assets);
  assert.equal(proof.edits.length, 2);
  assert.equal(proof.edits[0].start, base.text.indexOf(assets.embedded.toString()));
  assert.equal(proof.edits[1].replacementBytes < proof.edits[1].end - proof.edits[1].start, true);
});

test("an authored range crossing the helper prefix boundary is refused", () => {
  const { source, base, head } = fixture();
  const start = base.text.indexOf(assets.embedded.toString());
  base.mappings.push([start - 1, 2, 0, 1, 1, 0]);
  head.mappings.push([start - 1, 2, 0, 1, 1, 0]);
  assert.throws(() => relocatePacket(base, head, source, assets), /certified replacement/);
});

test("matching authored coordinates outside the source are refused independently of parity", () => {
  const { source, base, head } = fixture();
  base.mappings[1][2] = bytes(source) + 1;
  head.mappings[1][2] = bytes(source) + 1;
  assert.throws(() => relocatePacket(base, head, source, assets), /exceeds/);
});

test("every semantic, diagnostic, feature and byte difference remains observable", () => {
  const mutations = [
    (head: ReturnType<typeof fixture>["head"]) => {
      head.mappings[1][0]++;
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.mappings[1][2]++;
      head.mappings[1][3]--;
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.mappings[1][5] = 0;
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.semanticLinks[0].targetStart++;
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.diagnosticDirectives.directives[0][3]++;
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.diagnostics[0].messageText = "changed";
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.diagnostics.push({ ...head.diagnostics[0] });
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.text = head.text.replace("const result", "const altered");
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.text = head.text.replace("// @vize-map: 0:0", "// @vize-map: 1:0");
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.text = head.text.replace(
        /(\/\/ @vize-map: )\d+(?=:.*-> 0:\d+\nconst result)/u,
        "$1999999",
      );
    },
    (head: ReturnType<typeof fixture>["head"]) => {
      head.text = head.text.replace("  __EmitShape,", "  __EmitArgs,");
    },
  ];
  for (const mutate of mutations) {
    const { source, base, head } = fixture();
    mutate(head);
    assert.throws(
      () => relocatePacket(base, head, source, assets),
      { name: "AssertionError" },
      mutate.toString(),
    );
  }
});

test("all 228 original observations participate, including every unshared packet", () => {
  const { source, base, head } = fixture();
  const baseRows: Row[] = [],
    headRows: Row[] = [];
  for (let index = 0; index < 228; index++) {
    const shared = index < 96;
    const metadata = {
      case: String(index),
      newline: "\n",
      source,
      target: shared ? "content-mapper" : "checker",
      ...(shared ? { options: 8 } : {}),
    };
    baseRows.push({ ...metadata, packet: structuredClone(base) });
    headRows.push({ ...metadata, packet: structuredClone(shared ? head : base) });
  }
  assert.equal(comparePackets(baseRows, headRows, assets).unchangedPackets, 132);
  assert.equal(comparePackets(baseRows, structuredClone(baseRows), assets).unchangedPackets, 228);
  headRows[227].packet.diagnostics.push({ start: 0, length: 1 });
  assert.throws(() => comparePackets(baseRows, headRows, assets));
  assert.throws(() => comparePackets(baseRows, headRows.slice(1), assets));
});

test("canonical bodies and alias order are independently exact", (context) => {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-helper-parity-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const base = path.join(directory, "base"),
    head = path.join(directory, "head");
  const importPath =
    "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt";
  for (const destination of [base, head])
    for (const file of [...canonicalPaths, importPath]) {
      mkdirSync(path.dirname(path.join(destination, file)), { recursive: true });
      writeFileSync(path.join(destination, file), readFileSync(path.join(root, file)));
    }
  readAssets(base, head);
  const canonical = path.join(head, canonicalPaths[0]);
  writeFileSync(canonical, readFileSync(canonical, "utf8") + "\n");
  assert.throws(() => readAssets(base, head), /canonical helper body changed/);
  writeFileSync(canonical, readFileSync(path.join(base, canonicalPaths[0])));
  writeFileSync(
    path.join(head, importPath),
    assets.imported
      .toString()
      .replace("  __EmitShape,\n  __EmitArgs,", "  __EmitArgs,\n  __EmitShape,"),
  );
  assert.throws(() => readAssets(base, head), /alias order/);
});
