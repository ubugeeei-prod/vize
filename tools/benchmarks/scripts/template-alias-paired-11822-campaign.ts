import assert from "node:assert/strict";

export type Projection = { path: string; text: string; metadata: unknown };

/** Certify the expected listener-relation rewrite while retaining both whole projections. */
export function certifyListenerRelationProjections(before: Projection[], after: Projection[]) {
  assert.equal(after.length, before.length, "listener projection inventory changed");
  const inline = "[Extract<NonNullable<__H>, (...args: any) => any>] extends [never]";
  const shared = 'import("vize/virtual-ts").$ListenerArgs<';
  const count = (text: string, marker: string) => text.split(marker).length - 1;
  let changedDocuments = 0,
    removedInlineRelations = 0,
    sharedReferences = 0,
    beforeBytes = 0,
    afterBytes = 0;
  for (const [index, previous] of before.entries()) {
    const current = after[index]!;
    assert.equal(current.path, previous.path, "listener projection path order changed");
    beforeBytes += Buffer.byteLength(previous.text);
    afterBytes += Buffer.byteLength(current.text);
    if (current.text === previous.text) {
      assert.deepEqual(
        current.metadata,
        previous.metadata,
        "unchanged projection metadata changed",
      );
      continue;
    }
    changedDocuments++;
    const previousInline = count(previous.text, inline),
      currentInline = count(current.text, inline),
      currentShared = count(current.text, shared);
    assert.ok(previousInline > currentInline, "changed projection did not remove inline relations");
    assert.ok(currentShared > 0, "changed projection did not instantiate the shared relation");
    removedInlineRelations += previousInline - currentInline;
    sharedReferences += currentShared;
  }
  assert.ok(changedDocuments > 0, "listener campaign changed no projection");
  assert.ok(removedInlineRelations > 0 && sharedReferences > 0);
  return { changedDocuments, removedInlineRelations, sharedReferences, beforeBytes, afterBytes };
}

/** Admit exactly one production campaign and reject unrelated source changes. */
export function mapperCampaign(changed: string[]) {
  const dispatcher = changed.includes("crates/vize/src/commands/content_mapper/dispatch.rs");
  const listenerRelations = changed.includes(
    "crates/vize_canon/src/virtual_ts/scope/component_events/listener_relations.rs",
  );
  assert.notEqual(dispatcher, listenerRelations, "one mapper campaign production scope required");
  const listenerProduction = new Set([
    "crates/vize/src/commands/content_mapper/helper-abi.json",
    "crates/vize_canon/src/virtual_ts/generator.rs",
    "crates/vize_canon/src/virtual_ts/generator/script_module.rs",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/listener-args.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/declarations/shared-listener-args.d.ts.txt",
    "crates/vize_canon/src/virtual_ts/helpers/preamble.rs",
    "crates/vize_canon/src/virtual_ts/helpers/shared_types.rs",
    "crates/vize_canon/src/virtual_ts/scope/closures.rs",
    "crates/vize_canon/src/virtual_ts/scope/component_events.rs",
    "crates/vize_canon/src/virtual_ts/scope/component_events/listener_relations.rs",
    "crates/vize_canon/src/virtual_ts/scope/context.rs",
    "crates/vize_canon/src/virtual_ts/scope/event_scope/component_scope.rs",
    "npm/cli/package.json",
    "npm/cli/scripts/generate-content-mapper-types.ts",
    "npm/cli/scripts/listener-argument-exports.ts",
    "npm/cli/virtual-ts-listeners.d.ts",
    "npm/cli/virtual-ts.d.cts",
    "npm/cli/virtual-ts.d.mts",
    "npm/cli/virtual-ts.d.ts",
  ]);
  const campaign = dispatcher ? "dispatcher-admission" : "listener-argument-relations";
  assert.ok(
    changed.every(
      (path) =>
        path.startsWith(".github/") ||
        path.startsWith("tests/") ||
        path.startsWith("tools/") ||
        (path.startsWith("crates/") && path.includes("/tests/")) ||
        path.startsWith("crates/vize_canon/snapshots/") ||
        (path.startsWith("crates/vize_canon/src/") && path.includes("/snapshots/")) ||
        (dispatcher &&
          [
            "crates/vize/src/commands/content_mapper/dispatch.rs",
            "crates/vize/src/commands/content_mapper/dispatch/tests.rs",
          ].includes(path)) ||
        (listenerRelations &&
          (listenerProduction.has(path) ||
            (path.startsWith("crates/vize_canon/src/virtual_ts/") &&
              (path.includes("/tests/") ||
                path.includes("/current_output/") ||
                path.endsWith("/tests.rs"))))),
    ),
    `candidate contains production changes outside ${campaign}`,
  );
  return { campaign, dispatcher, listenerRelations } as const;
}
