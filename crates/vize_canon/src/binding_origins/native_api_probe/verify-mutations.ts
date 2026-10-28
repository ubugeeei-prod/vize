/** Selected-declaration mutation custody; no runtime provider classification. */
import assert from "node:assert/strict";
import { array, canonical, first, integer, object, text } from "./verify-codec.ts";
import type { ObjectValue } from "./verify-codec.ts";

/** Corsa2 numeric-wire snapshots serialize as numbers; Rust receipt handles are strings. */
export function snapshotIdentity(value: unknown): string {
  if (typeof value === "number") {
    const number = integer(value);
    assert.ok(number >= 0);
    return String(number);
  }
  const token = text(value);
  assert.match(token, /^(0|[1-9][0-9]*)$/u);
  return token;
}
function only(values: ObjectValue[]): ObjectValue {
  assert.equal(values.length, 1, "one exact current-snapshot request required");
  return first(values);
}
function declarationPaths(symbol: ObjectValue): string[] {
  const declarations = array(symbol.declarations);
  assert.ok(declarations.length > 0);
  return declarations.map((value) => {
    const handle = text(value);
    const firstDot = handle.indexOf(".");
    const secondDot = handle.indexOf(".", firstDot + 1);
    assert.ok(firstDot > 0 && secondDot > firstDot + 1);
    const file = handle.slice(secondDot + 1);
    assert.ok(file.startsWith("/"));
    return file;
  });
}
function symbolArgument(symbol: ObjectValue, version: string): unknown {
  return version === "7.1.0-beta" ? object(symbol.reference) : symbol.id;
}
function validateOwner(
  symbol: ObjectValue,
  version: string,
  snapshot: unknown,
  project: unknown,
): void {
  if (version === "7.0.2") {
    assert.ok(typeof symbol.id === "string" || Number.isSafeInteger(symbol.id));
    return;
  }
  assert.equal(version, "7.1.0-beta");
  const reference = object(symbol.reference);
  integer(reference.id);
  if (reference.kind === 0) {
    const file = object(reference.file);
    for (const key of [
      "path",
      "fileName",
      "contentHash",
      "parseOptionsKey",
      "scriptKind",
      "nodeId",
    ])
      assert.ok(Object.hasOwn(file, key));
  } else {
    assert.equal(reference.kind, 1, "unsupported native symbol owner");
    assert.equal(snapshotIdentity(reference.snapshot), snapshotIdentity(snapshot));
    assert.deepEqual(reference.project, project);
  }
}

export function verifyMutations(
  carrier: ObjectValue,
  requests: ObjectValue[],
  result: (request: ObjectValue) => unknown,
  version: string,
): ObjectValue | undefined {
  if (carrier.semanticMutationPhases === undefined) return;
  const phases = array(carrier.semanticMutationPhases).map(object);
  assert.equal(phases.length, 3);
  assert.deepEqual(
    phases.map((phase) => phase.label),
    ["initializer-vue", "initializer-custom", "initializer-repaired"],
  );
  const originalPhases = array(carrier.phases).map(object);
  const previous = originalPhases.map((value) => snapshotIdentity(value.snapshot));
  const originalFinal = originalPhases[2];
  assert.ok(originalFinal);
  const unchangedDiagnostics = array(originalFinal.semanticDiagnostics);
  const snapshots = phases.map((phase) => snapshotIdentity(phase.snapshot));
  assert.equal(new Set(snapshots).size, 3);
  assert.ok(snapshots.every((snapshot) => !previous.includes(snapshot)));
  assert.equal(phases[0]?.authoredSha256, phases[2]?.authoredSha256);
  assert.notEqual(phases[0]?.authoredSha256, phases[1]?.authoredSha256);
  const observations: unknown[] = [];
  for (const phase of phases) {
    const project = object(phase.project).id;
    const current = requests.filter((request) => {
      if (request.params === undefined || request.params === null) {
        assert.equal(request.method, "initialize", "only initialization omits parameters");
        return false;
      }
      const params = object(request.params);
      return (
        Object.hasOwn(params, "snapshot") &&
        snapshotIdentity(params.snapshot) === snapshotIdentity(phase.snapshot)
      );
    });
    const method = (name: string) => current.filter((request) => request.method === name);
    const queried = method("getSymbolAtPosition");
    assert.equal(queried.length, 3);
    const selectedRequest = only(
      queried.filter((request) => {
        const params = object(request.params);
        return params.file === phase.probePath && params.position === phase.probeUtf16Position;
      }),
    );
    const initializerRequests = queried.filter((request) => {
      const params = object(request.params);
      return (
        params.file === phase.authoredPath && params.position === phase.initializerUtf16Position
      );
    });
    assert.equal(initializerRequests.length, 2);
    const selected = object(result(selectedRequest));
    const initializer = object(result(first(initializerRequests)));
    const repeatedRequest = initializerRequests[1];
    assert.ok(repeatedRequest);
    assert.deepEqual(result(first(initializerRequests)), result(repeatedRequest));
    const aliases = method("getAliasedSymbol");
    const alias = (symbol: ObjectValue) =>
      only(
        aliases.filter(
          (request) =>
            canonical(object(request.params).symbol) === canonical(symbolArgument(symbol, version)),
        ),
      );
    const exported = object(result(alias(selected)));
    assert.equal(exported.name, "annotated");
    assert.ok(declarationPaths(exported).every((file) => file === phase.authoredPath));
    const typeRequest = only(method("getTypeOfSymbol"));
    assert.equal(
      canonical(object(typeRequest.params).symbol),
      canonical(symbolArgument(exported, version)),
    );
    const type = object(result(typeRequest));
    const ownerRequest = only(method("getSymbolOfType"));
    assert.equal(canonical(object(ownerRequest.params).type), canonical(type.id));
    const owner = object(result(ownerRequest));
    assert.equal(owner.name, "useTemplateRef");
    const providerDeclaration = object(phase.typeProvider).declaration;
    assert.ok(declarationPaths(owner).includes(text(providerDeclaration)));
    const isCustom = phase.label === "initializer-custom";
    let target: ObjectValue;
    if (isCustom) {
      assert.equal(aliases.length, 1);
      assert.equal(initializer.name, "custom");
      target = initializer;
      assert.ok(declarationPaths(target).every((file) => file === phase.authoredPath));
      assert.notEqual(
        canonical(symbolArgument(target, version)),
        canonical(symbolArgument(owner, version)),
      );
    } else {
      assert.equal(aliases.length, 2);
      assert.equal(initializer.name, "actual");
      target = object(result(alias(initializer)));
      assert.equal(
        canonical(symbolArgument(target, version)),
        canonical(symbolArgument(owner, version)),
      );
    }
    for (const symbol of [selected, exported, initializer, target, owner])
      validateOwner(symbol, version, phase.snapshot, project);
    for (const request of current) {
      const params = object(request.params);
      if (Object.hasOwn(params, "project")) assert.deepEqual(params.project, project);
    }
    const files = array(result(only(method("getSourceFileNames")))).map(text);
    assert.equal(files.length, phase.sourceInputManifestEntries);
    const syntax = method("getSyntacticDiagnostics");
    if (version === "7.0.2") {
      assert.deepEqual(
        syntax.map((request) => text(object(request.params).file)).sort(),
        [...files].sort(),
      );
    } else {
      assert.deepEqual(
        array(object(only(syntax).params).files)
          .map(text)
          .sort(),
        [...files].sort(),
      );
    }
    for (const request of syntax) {
      const diagnostics = result(request);
      assert.ok(diagnostics === null || array(diagnostics).length === 0);
    }
    const semantic = method("getSemanticDiagnostics");
    const expectedFiles = [text(phase.probePath), text(phase.authoredPath)].sort();
    if (version === "7.0.2") {
      assert.deepEqual(
        semantic.map((request) => text(object(request.params).file)).sort(),
        expectedFiles,
      );
    } else {
      assert.deepEqual(
        array(object(only(semantic).params).files)
          .map(text)
          .sort(),
        expectedFiles,
      );
    }
    const fullDiagnostics = semantic.flatMap((request) => {
      const value = result(request);
      return value === null ? [] : array(value);
    });
    assert.deepEqual(fullDiagnostics, phase.semanticDiagnostics);
    assert.deepEqual(fullDiagnostics, unchangedDiagnostics);
    observations.push({
      label: phase.label,
      snapshot: phase.snapshot,
      project,
      sourceSha256: phase.authoredSha256,
      selected,
      exported,
      initializer,
      target,
      type,
      typeOwner: owner,
      typeOrigin: "same Vue declaration across all revisions",
      valueOrigin: isCustom ? "authored custom function" : "authored Vue import",
      runtimeProducerAcknowledged: false,
      nativeProgramCount: null,
      nativeBindCount: null,
    });
  }
  return {
    schema: "vize.binding-origin.requested-mutation-wire.v1",
    observations,
    repeatedInitializerGroups: 3,
    sourceSha256IsNotNativeContentHash32: true,
    originClassifierQualified: false,
    hostWire: false,
    passed: true,
  };
}
