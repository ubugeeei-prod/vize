import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import {
  AUTHORED_SOURCE_SHAS,
  decodeFrozenExpected,
  loadFrozenAliasCases,
  rebindFrozenExpected,
  type FrozenPacketReceipt,
} from "./cases.ts";

const fixtureRoot = fileURLToPath(
  new URL("../../../../_fixtures/differential/lsp/type-alias-navigation/8011/", import.meta.url),
);
const historical = path.join(fixtureRoot, "supplemental/historical-c2bc");
const receipt = JSON.parse(readFileSync(path.join(historical, "receipt.json.txt"), "utf8")) as {
  packets: FrozenPacketReceipt[];
};
const sha = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");

test("authentic frozen authority contains all 26 transactions and six definition sessions", () => {
  const cases = loadFrozenAliasCases(fixtureRoot);
  assert.equal(cases.length, 32);
  assert.equal(cases.filter((item) => item.kind === "transaction").length, 26);
  assert.equal(cases.filter((item) => item.kind === "definitions").length, 6);
  assert.equal(new Set(cases.map((item) => item.context)).size, 32);
  for (const form of ["Plain", "Defaults", "Partial"]) {
    const count = form === "Defaults" ? 6 : 5;
    assert.equal(cases.filter((item) => item.form === form && item.newline === "\n").length, count);
    assert.equal(
      cases.filter((item) => item.form === form && item.newline === "\r\n").length,
      count,
    );
  }
  const original = cases.find((item) => item.form === "Plain" && item.newline === "\n")!;
  assert.equal(sha(Buffer.from(original.input)), AUTHORED_SOURCE_SHAS["E.vue.txt"]);
  assert.ok(
    Object.isFrozen(cases) && Object.isFrozen(original) && Object.isFrozen(original.expected),
  );
  assert.throws(() => Object.assign(original.expected, { unexpected: true }), TypeError);
});

test("whole expectation rebinding changes exactly the explicit authored URI and nothing else", () => {
  for (const item of loadFrozenAliasCases(fixtureRoot)) {
    const newUri = `file:///independent/${item.id}/E.vue`;
    const before = JSON.stringify(item.expected);
    const rebound = rebindFrozenExpected(item, { oldUri: item.authoredUri, newUri });
    assert.equal(JSON.stringify(rebound), before.replaceAll(item.authoredUri, newUri));
    assert.equal(JSON.stringify(item.expected), before);
    assert.ok(Object.isFrozen(rebound));
    assert.throws(() => rebindFrozenExpected(item, { oldUri: "file:///wrong/E.vue", newUri }));
  }
});

test("unknown expected URI values and WorkspaceEdit keys cannot be silently rebound", () => {
  const item = loadFrozenAliasCases(fixtureRoot).find(
    (candidate) => candidate.kind === "transaction",
  )!;
  const newUri = "file:///independent/E.vue";
  for (const extra of [
    { unexpected: { uri: "file:///foreign/E.vue" } },
    { unexpected: "untitled:foreign" },
    { unexpected: { uri: "unqualified" } },
    { unexpected: { changes: { "file:///foreign/E.vue": [] } } },
    { unexpected: `${item.authoredUri}?foreign` },
  ]) {
    const changed = { ...item, expected: { ...item.expected, ...extra } };
    assert.throws(
      () => rebindFrozenExpected(changed, { oldUri: item.authoredUri, newUri }),
      /unknown authored URI/,
    );
  }
});

test("gzip and raw identities fail independently before malformed JSON can be parsed", () => {
  const entry = receipt.packets[0];
  const compressed = readFileSync(path.join(historical, entry.file));
  const damaged = Buffer.from(compressed);
  damaged[damaged.length - 1] ^= 1;
  assert.throws(() => decodeFrozenExpected(damaged, entry), /gzip SHA custody/);
  assert.throws(
    () => decodeFrozenExpected(compressed, { ...entry, rawPacketSha256: "0".repeat(64) }),
    /raw packet SHA custody/,
  );
  const invalidGzip = Buffer.from("not a gzip stream");
  assert.throws(
    () => decodeFrozenExpected(invalidGzip, { ...entry, gzipSha256: sha(invalidGzip) }),
    /header|gzip|compression/i,
  );
  const invalidJson = gzipSync(Buffer.from("{"));
  assert.throws(
    () => decodeFrozenExpected(invalidJson, { ...entry, gzipSha256: sha(invalidJson) }),
    /raw packet SHA custody/,
  );
  assert.throws(
    () =>
      decodeFrozenExpected(invalidJson, {
        ...entry,
        gzipSha256: sha(invalidJson),
        rawPacketSha256: sha(Buffer.from("{")),
      }),
    SyntaxError,
  );
});

test("capture context is authenticated and the reader exposes expected without actual", () => {
  const entry = receipt.packets[0];
  const compressed = readFileSync(path.join(historical, entry.file));
  const decoded = decodeFrozenExpected(compressed, entry);
  assert.deepEqual(Object.keys(decoded), ["context", "initialFiles", "expected"]);
  assert.throws(
    () => decodeFrozenExpected(compressed, { ...entry, context: "alias unknown" }),
    /context receipt custody/,
  );
});

test("a changed independent source cannot inherit authority from matching provider packets", () => {
  const scratch = mkdtempSync(path.join(tmpdir(), "vize-frozen-alias-law-"));
  try {
    for (const file of Object.keys(AUTHORED_SOURCE_SHAS)) {
      const target = path.join(scratch, file);
      mkdirSync(path.dirname(target), { recursive: true });
      copyFileSync(path.join(fixtureRoot, file), target);
    }
    const target = path.join(scratch, "supplemental/E-defaults.vue.txt");
    writeFileSync(target, `${readFileSync(target, "utf8")}<!-- unauthorized -->\n`);
    assert.throws(() => loadFrozenAliasCases(scratch), /independent source SHA custody/);
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
});
