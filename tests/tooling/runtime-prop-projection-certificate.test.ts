import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test, type TestContext } from "node:test";
import {
  canonicalPaths,
  nativeAttributesPath,
  readAssets,
  relocatePacket,
  relocateVirtualPacket,
} from "../../tools/support/compat/content-mapper/constructor-projection-parity.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const previous =
  "type __RuntimePropValue<T> = T extends abstract new (...args: any[]) => infer V ? V : T extends (...args: any[]) => infer V ? V : never;";
const current =
  "type __RuntimePropValue<T> = T extends (...args: any[]) => infer V ? V : T extends abstract new (...args: any[]) => infer V ? V : never;";
const corePath = canonicalPaths[0];
assert.ok(corePath);
const currentCore = readFileSync(path.join(root, corePath), "utf8");
assert.equal(currentCore.split(current).length, 2);
const previousCore = currentCore.replace(current, previous);

function roots(context: TestContext) {
  const directory = mkdtempSync(path.join(os.tmpdir(), "runtime-prop-certificate-"));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const base = path.join(directory, "base"),
    head = path.join(directory, "head");
  for (const role of [base, head])
    for (const file of [
      ...canonicalPaths,
      nativeAttributesPath,
      "crates/vize_canon/src/virtual_ts/helpers/declarations/emit-overloads.d.ts.txt",
      "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
    ]) {
      const destination = path.join(role, file);
      mkdirSync(path.dirname(destination), { recursive: true });
      copyFileSync(path.join(root, file), destination);
    }
  writeFileSync(path.join(base, corePath), previousCore);
  return { base, head };
}

const source = "<template>雪🌸</template>";
const bytes = (text: string) => Buffer.byteLength(text);
function packet(helper: Buffer) {
  const prefix = "// prefix\n",
    body = "const value = '雪🌸';\n";
  const start = bytes(prefix) + helper.length,
    end = start + bytes(body);
  const text =
    prefix + helper.toString() + body + `// @vize-map: ${start}:${end} -> 0:${bytes(source)}\n`;
  return {
    text,
    mappings: [[start, bytes(body), 0, bytes(source), 1, 0]],
    semanticLinks: [
      { sourceStart: start, sourceLength: 5, targetStart: start + 6, targetLength: 5 },
    ],
    diagnosticDirectives: {
      unusedExpectDirectiveDiagnostics: [],
      directives: [
        [0, 0, 0, start, 0, 0],
        [0, 1, start, end, 0, 0],
        [0, 0, end, bytes(text), 0, 0],
      ],
    },
    diagnostics: [{ start: 0, length: 1, code: 2322, messageText: "retained authored error" }],
  };
}
function virtual(value: ReturnType<typeof packet>) {
  return {
    virtual_ts: value.text,
    virtual_ts_mappings: value.mappings.map(([start, length, authored, authoredLength]) => ({
      generated: { start, end: start + length },
      authored: { start: authored, end: authored + authoredLength },
      subSpans: [],
    })),
    virtual_ts_semantic_links: value.semanticLinks.map((link) => ({
      source: { start: link.sourceStart, end: link.sourceStart + link.sourceLength },
      target: { start: link.targetStart, end: link.targetStart + link.targetLength },
    })),
    diagnostics: value.diagnostics,
  };
}

function authoredInsideHelper(value: ReturnType<typeof packet>, start: number, length: number) {
  const body = value.diagnosticDirectives.directives.find((row) => row[1] > 0)!;
  return {
    ...value,
    mappings: [[start, length, 0, 1, 1, 0]],
    diagnosticDirectives: {
      unusedExpectDirectiveDiagnostics: [],
      directives: [
        [0, 0, 0, start, 0, 0],
        [0, 0, start + length, body[2], 0, 0],
        body,
        [0, 0, body[3], bytes(value.text), 0, 0],
      ],
    },
  };
}

test("the exact runtime-prop source transition certifies complete mapper and virtual packets", (context) => {
  const { base, head } = roots(context);
  const assets = readAssets(base, head);
  const core = assets.canonical[0];
  assert.ok(core);
  assert.equal(core.baseSha256, "1cb95f44b238332378bab53af58bbe0b95ad428d2a846f3136b81bfc3590cb98");
  assert.equal(core.headSha256, "3767bffe638c34126238eb7122ed70be0c5d1cbc8a79349aa10bdbc08f0820b5");
  assert.equal(assets.embedded.length, assets.headEmbedded.length);
  for (const replacement of [assets.imported, assets.headEmbedded]) {
    const before = packet(assets.embedded),
      after = packet(replacement);
    assert.ok(relocatePacket(before, after, source, assets, replacement).edits.length > 0);
    assert.ok(
      relocateVirtualPacket(virtual(before), virtual(after), source, assets, replacement).edits
        .length > 0,
    );
  }
});

test("runtime-prop certification rejects every unrelated asset byte and the reverse transition", (context) => {
  const { base, head } = roots(context);
  for (const mutated of [
    currentCore + "\n",
    currentCore.replace(current, ""),
    currentCore + current,
    currentCore.replace("type $IsAny<T>", "type $Foreign<T>"),
    currentCore.replace(current, current.replace("? V : T", "? any : T")),
  ]) {
    writeFileSync(path.join(head, corePath), mutated);
    assert.throws(() => readAssets(base, head), /canonical helper body changed/);
  }
  writeFileSync(path.join(head, corePath), previousCore);
  writeFileSync(path.join(base, corePath), currentCore);
  assert.throws(() => readAssets(base, head), /canonical helper body changed/);
  writeFileSync(path.join(base, corePath), previousCore);
  writeFileSync(path.join(head, corePath), currentCore);
  const other = canonicalPaths[1];
  assert.ok(other);
  writeFileSync(path.join(head, other), readFileSync(path.join(head, other), "utf8") + "\n");
  assert.throws(() => readAssets(base, head), /canonical helper body changed/);
});

test("certified runtime-prop changes preserve authored offsets, diagnostics and every other packet byte", (context) => {
  const { base, head } = roots(context);
  const assets = readAssets(base, head);
  for (const replacement of [assets.imported, assets.headEmbedded]) {
    const before = packet(assets.embedded),
      after = packet(replacement);
    for (const mutated of [
      { ...after, text: after.text + "// unexpected\n" },
      { ...after, mappings: [[after.mappings[0][0] + 1, 1, 0, 1, 1, 0]] },
      { ...after, diagnostics: [{ ...after.diagnostics[0], messageText: "lost authored error" }] },
    ]) {
      assert.throws(() => relocatePacket(before, mutated, source, assets, replacement));
      assert.throws(() =>
        relocateVirtualPacket(virtual(before), virtual(mutated), source, assets, replacement),
      );
    }
    const outside = (value: ReturnType<typeof packet>) => ({
      ...value,
      mappings: [[value.mappings[0][0], 1, bytes(source) + 1, 1, 1, 0]],
    });
    assert.throws(
      () => relocatePacket(outside(before), outside(after), source, assets, replacement),
      /exceeds/,
    );
    assert.throws(
      () =>
        relocateVirtualPacket(
          virtual(outside(before)),
          virtual(outside(after)),
          source,
          assets,
          replacement,
        ),
      /exceeds/,
    );
    const crosses = (value: ReturnType<typeof packet>) =>
      authoredInsideHelper(value, bytes("// prefix\n") - 1, 2);
    assert.throws(
      () => relocatePacket(crosses(before), crosses(after), source, assets, replacement),
      /certified replacement/,
    );
    assert.throws(
      () =>
        relocateVirtualPacket(
          virtual(crosses(before)),
          virtual(crosses(after)),
          source,
          assets,
          replacement,
        ),
      /certified replacement/,
    );
  }
});

test("pruned helper fragments certify only the same exact runtime-prop transition", (context) => {
  const { base, head } = roots(context);
  const assets = readAssets(base, head);
  const before = packet(Buffer.from(previous + "\n"));
  const after = packet(Buffer.from(current + "\n"));
  assert.equal(
    relocatePacket(before, after, source, assets, assets.headEmbedded).helperStart,
    null,
  );
  assert.equal(
    relocateVirtualPacket(virtual(before), virtual(after), source, assets, assets.headEmbedded)
      .helperStart,
    null,
  );
  for (const [oldText, newText] of [
    [previous + previous, current],
    [previous, current + current],
    [current, previous],
    ["", current],
    [previous, ""],
    [previous, current.replace("infer V", "infer Other")],
  ]) {
    const left = packet(Buffer.from(oldText)),
      right = packet(Buffer.from(newText));
    assert.throws(() => relocatePacket(left, right, source, assets, assets.headEmbedded));
    assert.throws(() =>
      relocateVirtualPacket(virtual(left), virtual(right), source, assets, assets.headEmbedded),
    );
  }
  const entersAlias = (value: ReturnType<typeof packet>) =>
    authoredInsideHelper(value, bytes("// prefix\n") + 5, 1);
  assert.throws(
    () =>
      relocatePacket(entersAlias(before), entersAlias(after), source, assets, assets.headEmbedded),
    /certified replacement/,
  );
  assert.throws(
    () =>
      relocateVirtualPacket(
        virtual(entersAlias(before)),
        virtual(entersAlias(after)),
        source,
        assets,
        assets.headEmbedded,
      ),
    /certified replacement/,
  );
});

test("embedded shared helpers retain their certified body and generated-only ignore envelope", (context) => {
  const { base, head } = roots(context);
  const assets = readAssets(base, head);
  const before = packet(assets.embedded),
    after = packet(assets.headEmbedded);
  for (const value of [before, after]) {
    value.text += assets.imported.toString();
    value.diagnosticDirectives.directives.at(-1)![3] = bytes(value.text);
  }
  assert.ok(relocatePacket(before, after, source, assets, assets.imported).edits.length > 0);
  const ignore = before.diagnosticDirectives.directives[0];
  for (const malformed of [
    [0, 1, ...ignore.slice(2)],
    [1, 0, ...ignore.slice(2)],
    [...ignore.slice(0, 4), 1, 0],
    [...ignore.slice(0, 5), 1],
    [0, 0, bytes("// prefix\n") + 1, ignore[3], 0, 0],
    [0, 0, 0, bytes("// prefix\n") + 1, 0, 0],
  ]) {
    const left = structuredClone(before),
      right = structuredClone(after);
    left.diagnosticDirectives.directives[0] = malformed;
    right.diagnosticDirectives.directives[0] = malformed;
    assert.throws(() => relocatePacket(left, right, source, assets, assets.imported));
  }
  const drift = structuredClone(after);
  drift.diagnosticDirectives.directives[0][3]++;
  assert.throws(() => relocatePacket(before, drift, source, assets, assets.imported));
});

test("the authenticated runtime-prop head refuses a completely missing producer population", (context) => {
  const { base, head } = roots(context);
  const assets = readAssets(base, head);
  const before = packet(assets.embedded),
    after = packet(assets.headEmbedded);
  const noGenerated = structuredClone(after);
  noGenerated.diagnosticDirectives.directives = noGenerated.diagnosticDirectives.directives.filter(
    (row) => row[0] !== 0 || row[1] !== 0,
  );
  assert.throws(
    () => relocatePacket(before, noGenerated, source, assets, assets.headEmbedded),
    /synthetic ignores/,
  );
  const historical = structuredClone(before);
  historical.diagnosticDirectives.directives = historical.diagnosticDirectives.directives.filter(
    (row) => row[0] !== 0 || row[1] !== 0,
  );
  assert.doesNotThrow(() => relocatePacket(historical, after, source, assets, assets.headEmbedded));
  assert.throws(
    () => relocatePacket(historical, noGenerated, source, assets, assets.headEmbedded),
    /synthetic ignores/,
  );
  for (const directives of [
    after.diagnosticDirectives.directives.slice(1),
    [after.diagnosticDirectives.directives[0], ...after.diagnosticDirectives.directives],
  ]) {
    const malformed = structuredClone(after);
    malformed.diagnosticDirectives.directives = directives;
    assert.throws(
      () => relocatePacket(before, malformed, source, assets, assets.headEmbedded),
      /synthetic ignores/,
    );
  }
  const absent = { ...after, diagnosticDirectives: undefined };
  assert.throws(
    () => relocatePacket(before, absent, source, assets, assets.headEmbedded),
    /synthetic ignores/,
  );
});
