// One reviewed URI boundary transition; historical ABI custody remains required.
import assert from "node:assert/strict";
import { gunzipSync } from "node:zlib";
import { createHash } from "node:crypto";
import { nativeBetaDtoProducerCustody } from "./native-beta-dto-fixtures.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const NATIVE_DOCUMENT_IDENTITY_ID = "typechecker/protocol/native-document-identity-11586";
const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const registrationSha256 = "cbe3def290a060a0486c3709e37d004a9003ae9e83c049242886e04fabef93cc";
const componentSource = "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs";
const canonical = (value: unknown): string =>
  JSON.stringify(value, (_, item: unknown) =>
    item && typeof item === "object" && !Array.isArray(item)
      ? Object.fromEntries(
          Object.entries(item).sort(([left], [right]) => left.localeCompare(right)),
        )
      : item,
  );
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function pinned(read: ReadFile, side: "base" | "head", path: unknown, expected: unknown) {
  assert.equal(typeof path, "string");
  assert.equal(typeof expected, "string");
  const bytes = read(side, String(path));
  assert.ok(bytes, `missing ${side} identity source: ${path}`);
  assert.equal(hash(bytes), expected, `changed ${side} identity source: ${path}`);
  return bytes;
}
function consumer(read: ReadFile, value: unknown) {
  const test = object(value);
  const source = pinned(read, "head", test.source, test.sha256).toString("utf8");
  assert.equal(typeof test.selector, "string");
  const name = String(test.selector).split("::").at(-1)!;
  assert.match(source, new RegExp(`#\\[test\\][\\s\\S]*?\\bfn ${name}\\(`));
  if (test.input !== null) {
    assert.equal(typeof test.input, "string");
    assert.ok(source.includes("include_bytes!(concat!("));
    assert.ok(source.includes(String(test.input)), "compiled consumer must read its pinned input");
  }
}
function preserveHistoricalAbi(read: ReadFile, value: unknown) {
  const legacy = object(value);
  assert.equal(legacy.id, "typechecker/protocol/native-beta-diagnostic-dto-10731");
  const registrations = (["base", "head"] as const).map((side) => {
    const manifest = JSON.parse(read(side, manifestPath)!.toString("utf8"));
    const rows = manifest.protocolCases.filter((row: { id: string }) => row.id === legacy.id);
    assert.equal(rows.length, 1);
    assert.equal(hash(Buffer.from(canonical(rows[0]))), legacy.registrationSha256);
    return rows[0];
  });
  assert.deepEqual(registrations[0], registrations[1], "historical ABI row must remain unchanged");
  assert.ok(Array.isArray(legacy.inputs) && legacy.inputs.length === 4);
  assert.deepEqual(
    legacy.inputs,
    registrations[0].inputs.map((input: { path: string; sha256: string }) => ({
      path: "tests/_fixtures/differential/typechecker/" + input.path,
      sha256: input.sha256,
    })),
  );
  for (const value of legacy.inputs) {
    const input = object(value);
    assert.ok(
      pinned(read, "base", input.path, input.sha256).equals(
        pinned(read, "head", input.path, input.sha256),
      ),
    );
  }
  assert.ok(Array.isArray(legacy.tests) && legacy.tests.length === 5);
  assert.deepEqual(
    legacy.tests.map((value) => object(value).selector),
    registrations[0].tests.map((test: { selector: string }) => test.selector),
  );
  legacy.tests.forEach((value) => consumer(read, value));
  const workflow = object(legacy.workflow);
  assert.ok(
    pinned(read, "base", workflow.path, workflow.sha256).equals(
      pinned(read, "head", workflow.path, workflow.sha256),
    ),
  );
  const producers = nativeBetaDtoProducerCustody(read);
  assert.equal(
    producers.length,
    5,
    "actual four-role A/B workflow and all typed producers remain owned",
  );
}

/** A finite source/input registration supplies no native or performance qualification. */
export function newNativeDocumentIdentityCases(
  previous: unknown,
  current: unknown,
  read: ReadFile,
) {
  const result = { ids: [] as string[], sources: new Set<string>() };
  if (current === undefined) return result;
  assert.ok(previous === undefined || Array.isArray(previous));
  assert.ok(Array.isArray(current) && current.length === 1);
  const row = object(current[0]);
  assert.equal(row.id, NATIVE_DOCUMENT_IDENTITY_ID);
  if (Array.isArray(previous) && previous.length) {
    assert.deepEqual(current, previous, "registered identity history must remain unchanged");
    return result;
  }
  assert.equal(
    hash(Buffer.from(canonical(row))),
    registrationSha256,
    "unregistered identity transition",
  );
  assert.equal(row.state, "active");
  assert.equal(row.adapter, "native-document-identity-current-v1");
  assert.deepEqual(row.targets, ["native-document-identity"]);
  assert.equal(row.nativeQualification, false);
  assert.ok(Array.isArray(row.sources) && row.sources.length === 16);
  const archive = object(row.sourceArchive);
  const originals = JSON.parse(
    gunzipSync(pinned(read, "head", archive.path, archive.sha256)).toString("utf8"),
  );
  assert.deepEqual(
    Object.keys(originals),
    row.sources.map((value) => object(value).path),
  );
  for (const value of row.sources) {
    const source = object(value);
    const before = pinned(read, "base", source.path, source.before);
    assert.ok(before.equals(Buffer.from(originals[String(source.path)], "base64")));
    pinned(read, "head", source.path, source.after);
    result.sources.add(String(source.path));
  }
  assert.ok(result.sources.has(componentSource));
  assert.ok(Array.isArray(row.inputs) && row.inputs.length === 1);
  for (const value of row.inputs) {
    const input = object(value);
    const fixture = JSON.parse(pinned(read, "head", input.path, input.sha256).toString("utf8"));
    assert.equal(fixture.schema, 1);
    assert.equal(fixture.id, NATIVE_DOCUMENT_IDENTITY_ID);
    assert.deepEqual(
      fixture.documents.map((item: { name: string }) => item.name),
      ["App", "Child", "Leaf"],
    );
    assert.equal(fixture.methods.length, 5);
  }
  assert.ok(Array.isArray(row.tests) && row.tests.length === 1);
  row.tests.forEach((value) => consumer(read, value));
  preserveHistoricalAbi(read, row.legacyProtocol);
  result.ids.push(NATIVE_DOCUMENT_IDENTITY_ID);
  return result;
}
