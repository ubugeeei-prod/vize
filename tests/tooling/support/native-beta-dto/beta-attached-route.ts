import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { isDeepStrictEqual } from "node:util";
import { verifyNativeRelations, type WatcherContext } from "./stable-watcher-boundary.ts";

type Packet = Record<string, unknown>;
export interface AttachedOwner {
  nativeOwner: number;
  client: readonly unknown[];
  server: readonly unknown[];
  apiClient: readonly unknown[];
  apiServer: readonly unknown[];
}
const configurationMethods = new Set([
  "initialize",
  "parseConfigFile",
  "createSnapshot",
  "updateSnapshot",
  "describeCapabilities",
  "release",
]);
const semanticMethods = [
  "textDocument/diagnostic",
  "textDocument/hover",
  "textDocument/references",
  "textDocument/definition",
  "textDocument/completion",
  "textDocument/rename",
];
function object(value: unknown): Packet {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Packet;
}
function id(value: unknown) {
  assert.ok(
    typeof value === "string" || (typeof value === "number" && Number.isSafeInteger(value)),
  );
  return `${typeof value}:${value}`;
}
function canonical(value: unknown): string {
  if (Array.isArray(value)) return "[" + value.map(canonical).join(",") + "]";
  if (value !== null && typeof value === "object")
    return (
      "{" +
      Object.entries(value)
        .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0))
        .map(([key, field]) => JSON.stringify(key) + ":" + canonical(field))
        .join(",") +
      "}"
    );
  assert.ok(
    value === null ||
      typeof value === "string" ||
      typeof value === "boolean" ||
      (typeof value === "number" && Number.isFinite(value)),
  );
  return JSON.stringify(value);
}
function sameCompletion(actual: unknown, expected: unknown): boolean {
  const left = object(actual),
    right = object(expected);
  assert.ok(Array.isArray(left.items) && Array.isArray(right.items));
  // Only Go's top-level item order varies. Keep every item field, nested array
  // order, list metadata and duplicate count; retained raw packets stay intact.
  return isDeepStrictEqual(
    { ...left, items: left.items.map(canonical).sort() },
    { ...right, items: right.items.map(canonical).sort() },
  );
}

/** Configuration attachments do not attest which transport served semantics. */
export function verifyBetaAttachedRoute(
  owners: readonly AttachedOwner[],
  context: WatcherContext,
  expectedSource: string,
  attachedRoot: string,
  sourceRows: readonly unknown[],
) {
  assert.equal(context.role, "after");
  assert.equal(context.sdkVersion, "7.1.0-beta");
  assert.equal(
    context.sdkSha256,
    "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
  assert.match(expectedSource, /^[a-f0-9]{40}$/u);
  assert.equal(context.source, expectedSource);
  assert.ok(context.projectRoots.includes(attachedRoot));
  assert.ok(
    path.posix.isAbsolute(attachedRoot) && path.posix.normalize(attachedRoot) === attachedRoot,
  );
  const phases = sourceRows.map(object);
  assert.deepEqual(
    phases.map((row) => row.phase),
    ["valid", "dependency-error", "dependency-repair", "dependency-close"],
  );
  for (const row of phases) {
    assert.equal(row.apiEligible, false);
    assert.equal(object(row.nativePull).kind, "full");
    assert.deepEqual(object(row.selected).complete, object(row.nativePull).items);
  }
  const mainUris = new Set([
    pathToFileURL(attachedRoot + "/main.ts").href,
    pathToFileURL(attachedRoot + "/node_modules/.vize/corsa-overlay/main.ts").href,
  ]);
  const ownerIds = owners.map((owner) => owner.nativeOwner);
  assert.ok(
    ownerIds.length > 0 && ownerIds.every((owner) => Number.isSafeInteger(owner) && owner > 0),
  );
  assert.equal(new Set(ownerIds).size, ownerIds.length);
  const evidence = owners.map((owner) => {
    assert.equal(verifyNativeRelations(owner.client, owner.server, context), null);
    assert.equal(verifyNativeRelations(owner.apiClient, owner.apiServer, context), null);
    const apiRequests = owner.apiClient.map(object).filter((row) => "id" in row && "method" in row);
    for (const request of apiRequests) {
      assert.equal(typeof request.method, "string");
      assert.ok(
        configurationMethods.has(request.method as string),
        "attached beta API may only configure and own snapshots",
      );
    }
    const replies = new Map(
      owner.server
        .map(object)
        .filter((row) => "id" in row && !("method" in row))
        .map((row) => [id(row.id), row]),
    );
    const calls = owner.client.map(object).filter((row) => {
      if (!("method" in row)) return false;
      assert.ok(typeof row.method === "string");
      return semanticMethods.includes(row.method);
    });
    const packets = calls.map((request) => {
      const uri = object(object(request.params).textDocument).uri;
      assert.equal(typeof uri, "string");
      const filename = fileURLToPath(uri as string);
      assert.equal(pathToFileURL(filename).href, uri);
      assert.ok(
        filename.startsWith(attachedRoot + "/"),
        "semantic request belongs to another project",
      );
      const response = replies.get(id(request.id));
      assert.ok(
        response && "result" in response && !("error" in response),
        "whole successful semantic LSP reply required",
      );
      return { request, response, uri };
    });
    const main = packets.filter((packet) => mainUris.has(packet.uri as string));
    const counts = Object.fromEntries(
      semanticMethods.map((method) => [
        method,
        main.filter((packet) => packet.request.method === method).length,
      ]),
    );
    const complete = semanticMethods.every(
      (method) => counts[method]! >= phases.length * (method === "textDocument/diagnostic" ? 3 : 2),
    );
    if (complete) {
      const order = [
        semanticMethods[0],
        semanticMethods[0],
        semanticMethods[0],
        ...semanticMethods.slice(1).flatMap((method) => [method, method]),
      ];
      assert.equal(
        main.length,
        phases.length * order.length,
        "complete distinct phase vectors required",
      );
      for (const [phaseIndex, phase] of phases.entries()) {
        const packets = main.slice(phaseIndex * order.length, (phaseIndex + 1) * order.length);
        assert.deepEqual(
          packets.map((packet) => packet.request.method),
          order,
          "raw semantic phase ordering differs",
        );
        const expected = [
          phase.nativePull,
          phase.nativePull,
          phase.nativePull,
          phase.hover,
          phase.hover,
          phase.references,
          phase.directReferences,
          phase.definition,
          phase.directDefinition,
          phase.completion,
          phase.directCompletion,
          object(phase.rename).Ok,
          phase.directRename,
        ];
        for (const [index, value] of expected.entries()) {
          assert.notEqual(value, undefined, "complete retained source phase output required");
          assert.ok(
            order[index] === "textDocument/completion"
              ? sameCompletion(packets[index]!.response.result, value)
              : isDeepStrictEqual(packets[index]!.response.result, value),
            `whole raw semantic result differs for phase ${phaseIndex}, query ${index}`,
          );
        }
      }
    }
    return {
      nativeOwner: owner.nativeOwner,
      apiRequests,
      apiResponses: owner.apiServer,
      semanticPackets: packets,
      counts,
      complete,
    };
  });
  assert.ok(
    evidence.some((owner) => owner.complete),
    "one native owner must serve all four phases through complete LSP query pairs",
  );
  return {
    schema: "vize.nativeBetaAttached.semanticRoute.v1",
    context,
    sourcePhases: sourceRows,
    owners: evidence,
    scope:
      "Complete semantic LSP pairs and configuration-only API pairs; an API attachment is not semantic-route evidence.",
  };
}
