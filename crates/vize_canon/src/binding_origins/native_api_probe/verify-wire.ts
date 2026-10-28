/** Complete pre-normalization JSON-RPC custody; no product origin classification. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { array, canonical, first, frames, integer, object, text } from "./verify-codec.ts";
import { strictJson } from "./verify-json.ts";
import { verifyMutations } from "./verify-mutations.ts";
import type { ObjectValue } from "./verify-codec.ts";

type Version = "7.0.2" | "7.1.0-beta";
type Role = "import" | "function";
const images: Record<Version, string> = {
  "7.0.2": "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
  "7.1.0-beta": "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
};
const tags: Record<Version, Record<Role, number>> = {
  "7.0.2": { import: 277, function: 263 },
  "7.1.0-beta": { import: 278, function: 264 },
};
const symbolMethods = new Set([
  "getSymbolAtPosition",
  "getAliasedSymbol",
  "getTypeOfSymbol",
  "getSymbolOfType",
]);
function isVersion(value: string): value is Version {
  return value === "7.0.2" || value === "7.1.0-beta";
}
function referenceParts(handle: string): [string, string, string] | undefined {
  const firstDot = handle.indexOf(".");
  const secondDot = handle.indexOf(".", firstDot + 1);
  if (firstDot < 1 || secondDot <= firstDot + 1) return;
  return [
    handle.slice(0, firstDot),
    handle.slice(firstDot + 1, secondDot),
    handle.slice(secondDot + 1),
  ];
}
export function fixtureReference(
  handle: string,
  version: string,
  role: Role,
  image: string,
): boolean {
  if (!isVersion(version) || images[version] !== image) return false;
  const parts = referenceParts(handle);
  return (
    parts !== undefined &&
    /^\d+$/u.test(parts[0]) &&
    /^\d+$/u.test(parts[1]) &&
    parts[2].startsWith("/") &&
    Number(parts[1]) === tags[version][role]
  );
}
function id(value: unknown): string | number {
  if (typeof value === "string") return value;
  return integer(value);
}
const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function verifyBuffers(
  clientBytes: Buffer,
  nativeBytes: Buffer,
  carrierValue: unknown,
  versionValue: string,
  emit: (name: string, value: unknown) => void,
): ObjectValue {
  assert.ok(isVersion(versionValue), "unsupported SDK version");
  const version = versionValue;
  const client = frames(clientBytes),
    native = frames(nativeBytes);
  const requests = client.filter(
    (packet) => typeof packet.method === "string" && Object.hasOwn(packet, "id"),
  );
  const responses = native.filter(
    (packet) => Object.hasOwn(packet, "id") && !Object.hasOwn(packet, "method"),
  );
  assert.equal(requests.length, client.length);
  assert.equal(responses.length, native.length);
  const symbolIDs = new Set(
    requests
      .filter((packet) => symbolMethods.has(text(packet.method)))
      .map((packet) => id(packet.id)),
  );
  emit("raw-symbol-candidates.json", {
    requests: requests.filter((packet) => symbolIDs.has(id(packet.id))),
    responses: responses.filter((packet) => symbolIDs.has(id(packet.id))),
  });
  const requestIDs = requests.map((packet) => id(packet.id));
  assert.equal(new Set(requestIDs).size, requestIDs.length);
  assert.equal(responses.length, requests.length);
  const byID = new Map<string | number, ObjectValue>();
  for (const packet of responses) {
    const key = id(packet.id);
    assert.ok(!byID.has(key), "duplicate full response");
    assert.ok(
      Object.hasOwn(packet, "result") && !Object.hasOwn(packet, "error"),
      "native API error",
    );
    byID.set(key, packet);
  }
  for (const key of requestIDs) assert.ok(byID.has(key), "missing full response");
  const result = (request: ObjectValue): unknown => {
    const response = byID.get(id(request.id));
    assert.ok(response);
    return response.result;
  };
  const counts = new Map<string, number>();
  for (const request of requests) {
    const method = text(request.method);
    counts.set(method, (counts.get(method) ?? 0) + 1);
  }
  const carrier = object(carrierValue);
  assert.equal(carrier.backendSha256, images[version]);
  const phases = array(carrier.phases).map(object);
  assert.equal(phases.length, 3);
  assert.deepEqual(
    phases.map((phase) => phase.syntaxComplete),
    [true, false, true],
  );
  const grouped = new Map<string, ObjectValue[]>();
  for (const request of requests.filter((packet) => packet.method === "getSymbolAtPosition")) {
    const key = canonical(object(request.params));
    const group = grouped.get(key) ?? [];
    group.push(request);
    grouped.set(key, group);
  }
  const baselines = Array.from(grouped.values()).filter((group) => group.length === 3);
  assert.equal(baselines.length, 3);
  const hasMutation = carrier.semanticMutationPhases !== undefined;
  for (const [method, expected] of [
    ["getSymbolAtPosition", hasMutation ? 63 : 54],
    ["getAliasedSymbol", hasMutation ? 20 : 15],
    ["getTypeOfSymbol", hasMutation ? 6 : 3],
    ["getSymbolOfType", hasMutation ? 6 : 3],
    ["getSourceFileNames", hasMutation ? 6 : 3],
  ] as const)
    assert.equal(counts.get(method), expected);
  for (const request of requests) {
    const value = result(request);
    if (
      !["getSymbolAtPosition", "getAliasedSymbol", "getSymbolOfType"].includes(
        text(request.method),
      ) ||
      value === null
    )
      continue;
    const symbol = object(value);
    const declarations = array(symbol.declarations);
    assert.ok(declarations.length > 0);
    for (const handle of declarations) assert.ok(text(handle).length > 0);
    if (version === "7.1.0-beta") {
      const reference = object(symbol.reference);
      integer(reference.id);
      if (reference.kind === 0) {
        const file = object(reference.file);
        for (const key of [
          "fileName",
          "path",
          "contentHash",
          "parseOptionsKey",
          "scriptKind",
          "nodeId",
        ])
          assert.ok(Object.hasOwn(file, key));
      } else {
        assert.equal(reference.kind, 1);
        assert.ok(Object.hasOwn(reference, "snapshot") && Object.hasOwn(reference, "project"));
      }
    } else {
      id(symbol.id);
    }
  }
  const mutation = verifyMutations(carrier, requests, result, version);
  if (mutation !== undefined) emit("semantic-mutation-wire.json", mutation);
  const controls: unknown[] = [];
  for (const group of baselines) {
    const values = group.map((request) => result(request));
    const selected = object(first(values));
    for (const value of values) assert.deepEqual(value, selected);
    assert.equal(selected.name, "actual");
    const argument = version === "7.1.0-beta" ? selected.reference : selected.id;
    const snapshot = object(first(group).params).snapshot;
    const aliases = requests.filter((request) => {
      if (request.method !== "getAliasedSymbol") return false;
      const params = object(request.params);
      return canonical(params.symbol) === canonical(argument) && params.snapshot === snapshot;
    });
    assert.equal(aliases.length, 1);
    const target = object(result(first(aliases)));
    assert.equal(target.name, "useTemplateRef");
    for (const [role, symbol] of [
      ["import", selected],
      ["function", target],
    ] as const) {
      const handle = text(first(array(symbol.declarations)));
      assert.ok(fixtureReference(handle, version, role, text(carrier.backendSha256)));
      const parts = referenceParts(handle);
      assert.ok(parts);
      const other: Version = version === "7.0.2" ? "7.1.0-beta" : "7.0.2";
      const poisoned = [parts[0], String(tags[other][role]), parts[2]].join(".");
      assert.equal(fixtureReference(poisoned, version, role, text(carrier.backendSha256)), false);
      assert.equal(
        fixtureReference(
          [parts[0], "99999", parts[2]].join("."),
          version,
          role,
          text(carrier.backendSha256),
        ),
        false,
      );
      assert.equal(fixtureReference(handle, "unknown", role, text(carrier.backendSha256)), false);
      assert.equal(fixtureReference(handle, version, role, "0".repeat(64)), false);
      controls.push({ role, actual: handle, oppositeVersionTag: poisoned, rejected: true });
    }
  }
  const retained = requests
    .filter((packet) => symbolMethods.has(text(packet.method)))
    .map((request) => ({ request, response: byID.get(id(request.id)) }));
  emit("full-wire-symbol-packets.json", retained);
  assert.equal(carrier.forwardedClientBytes, clientBytes.length);
  assert.equal(carrier.observedNativeStdoutBytes, nativeBytes.length);
  const receipt = {
    schema: "vize.binding-origin.native-api-wire.v2",
    version,
    scope:
      "test Unix relay to standalone native stdio; product LSP socket and origin host wiring unqualified",
    actualRequestCounts: Object.fromEntries(counts),
    actualResponseCount: responses.length,
    clientWireBytes: clientBytes.length,
    clientWireSha256: sha(clientBytes),
    nativeWireBytes: nativeBytes.length,
    nativeWireSha256: sha(nativeBytes),
    sameSnapshotRepeatGroups: 3,
    ...(mutation === undefined ? {} : { requestedMutation: mutation }),
    rawSymbolPacketsRetained: retained.length,
    versionBoundFixtureRefControls: controls,
    nodeRefTagIsNotPortableASTSyntaxKind: true,
    nativeProgramCount: null,
    nativeBindCount: null,
    originClassifierQualified: false,
    hostWire: false,
    passed: true,
  };
  emit("full-wire-receipt.json", receipt);
  return receipt;
}

export function verify(directory: string, version: string): ObjectValue {
  return verifyBuffers(
    readFileSync(join(directory, "client-to-native.wire.bin")),
    readFileSync(join(directory, "native-to-client.wire.bin")),
    strictJson(readFileSync(join(directory, "carrier-receipt.json"), "utf8")),
    version,
    (name, value) => writeFileSync(join(directory, name), JSON.stringify(value, null, 2) + "\n"),
  );
}
const entry = process.argv[1];
if (entry !== undefined && import.meta.url === pathToFileURL(entry).href) {
  const directory = process.argv[2],
    version = process.argv[3];
  if (directory === undefined || version === undefined)
    throw new Error("usage: verify-wire.ts DIRECTORY VERSION");
  console.log(JSON.stringify(verify(directory, version)));
}
