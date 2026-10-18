import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { readLspSourceFiles } from "../../../tools/support/release/public_acceptance/lsp_source.ts";

// Frozen historical path inventories reproduce the actual 467 -> 471 support
// addition. Inert file bodies and temporary Git trees do not execute a product.
const original467 = [
  "crates/vize/tests/fixtures/content_mapper_project/tsconfig.json",
  "crates/vize/tests/lsp_bound_event_casing_cli.rs",
  "crates/vize/tests/lsp_bound_event_casing_cli/application.rs",
  "crates/vize/tests/lsp_bound_event_casing_cli/session.rs",
  "crates/vize/tests/lsp_named_model_transactions_cli.rs",
  "crates/vize/tests/lsp_named_model_transactions_cli/contracts.rs",
  "crates/vize/tests/lsp_named_model_transactions_cli/session.rs",
  "crates/vize/tests/support/lsp_process.rs",
  "crates/vize/tests/support/lsp_process/capture.rs",
  "crates/vize/tests/support/lsp_process/evidence.rs",
  "crates/vize/tests/support/lsp_process/passive.rs",
  "crates/vize/tests/support/lsp_process/protocol.rs",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/README.md",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/call-signature/updated-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/cases.json.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/generic/updated-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/record/updated-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-array/updated-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/runtime-object/updated-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/README.md",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/author_contracts.py",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/cases.json.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/input-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/input-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/input-Parent.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/updated-Child.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/updated-Other.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/updated-Parent.vue.txt",
];
const atomic471 = [
  "crates/vize/tests/lsp_component_attribute_authority_cli.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/attribute_controls.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/native_probe.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/original_reads.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/cold_collision.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/diagnostic_witness.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/importer_controls.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/original.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/runtime_distribution.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/runtime_distribution/assets.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/safe_update.rs",
  "crates/vize/tests/lsp_library_rename_refusal_cli/safety_controls.rs",
  "tests/_fixtures/differential/lsp/event-rename/8010/App.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/8010/Toggle.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/8010/UpdatedApp.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt",
  "tests/_fixtures/differential/lsp/event-rename/8010/original-report.md",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.439.0-event-slot60/README.md",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.439.0-event-slot60/complete-evidence.tar.gz",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.439.0-event-slot60/receipt.json.txt",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.440.0-event-slot60/README.md",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.440.0-event-slot60/complete-evidence.tar.gz",
  "tests/_fixtures/differential/lsp/event-rename/8010/supplemental/public-0.440.0-event-slot60/receipt.json.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/App.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/Toggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/original-report.md",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/reported-native-edits.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/README.md",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/ambiguous-bound-emitter/Toggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/cold-value-collision/Toggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/cold-value-collision/UpdatedToggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/partial-application/CallOnlyToggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/partial-application/MissingCallToggle.vue.txt",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.alias.after.vue.txt",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.flip.after.vue.txt",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Probe.vue.txt",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/Toggle.mutable.vue.txt",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/historical-H.json",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/input.json",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/lib.custom.d.ts.txt",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/App.vue.txt",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/Classy.vue.txt",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/Item.vue.txt",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/tsconfig.json.txt",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/types.ts.txt",
];
const required = [
  "crates/vize/tests/lsp_named_model_transactions_cli.rs",
  "crates/vize/tests/lsp_named_model_transactions_cli/",
  "crates/vize/tests/lsp_bound_event_casing_cli.rs",
  "crates/vize/tests/lsp_bound_event_casing_cli/",
  "crates/vize/tests/support/lsp_process.rs",
  "crates/vize/tests/support/lsp_process/",
  "crates/vize/tests/fixtures/content_mapper_project/tsconfig.json",
  "tests/_fixtures/differential/lsp/event-rename/4075/named-model/",
  "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/",
  "crates/vize/tests/lsp_component_attribute_authority_cli.rs",
  "crates/vize/tests/lsp_component_attribute_authority_cli/",
  "crates/vize/tests/lsp_library_rename_refusal_cli/",
  "tests/_fixtures/differential/lsp/rename-source-authority/8009/",
  "tests/_fixtures/differential/lsp/rename-library-refusal/8010/",
  "tests/_fixtures/differential/lsp/event-rename/8010/",
  "tests/_fixtures/differential/lsp/rename-runtime-distribution/8010/",
];

function fixture(t: { after: (fn: () => void) => void }) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-source-closure-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const add = (file: string) => {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), `inert source ${file}\n`);
    git("add", "--", file);
  };
  git("init", "-q");
  const files = [...new Set([...original467, ...atomic471])];
  for (const file of files) add(file);
  return { root, git, add, files, tree: git("write-tree") };
}

test("historical 53 and actual 54 retain the complete tracked source, including future helpers", (t) => {
  const f = fixture(t);
  assert.equal(original467.length, 53, "historical fixture identity, not runtime admission");
  assert.deepEqual(readLspSourceFiles(f.root, f.tree), {
    originalFiles: [...original467].sort(),
    atomicFiles: [...atomic471].sort(),
  });
  const added = "crates/vize/tests/support/lsp_process/process_state.rs";
  f.add(added);
  const actual471 = f.git("write-tree");
  const original471 = [...original467, added].sort();
  assert.equal(original471.length, 54, "actual failed release path inventory");
  assert.deepEqual(readLspSourceFiles(f.root, actual471).originalFiles, original471);
  const next = "crates/vize/tests/support/lsp_process/future helper.rs";
  const nextAtomic = "crates/vize/tests/lsp_component_attribute_authority_cli/future.rs";
  f.add(next);
  f.add(nextAtomic);
  const nextTree = f.git("write-tree");
  assert.deepEqual(readLspSourceFiles(f.root, nextTree), {
    originalFiles: [...original471, next].sort(),
    atomicFiles: [...atomic471, nextAtomic].sort(),
  });
  // An untracked/working-tree change is never substituted for frozen H objects.
  f.add("outside-the-fixed-source-prefix.rs");
  fs.rmSync(path.join(f.root, added));
  assert.deepEqual(readLspSourceFiles(f.root, actual471).originalFiles, original471);
});

test("every required target, support and fixture prefix must remain present in H", (t) => {
  const f = fixture(t);
  for (const prefix of required) {
    f.git("read-tree", f.tree);
    const removed = f.files.filter((file) =>
      prefix.endsWith("/") ? file.startsWith(prefix) : file === prefix,
    );
    assert.ok(removed.length, `negative control must actually remove ${prefix}`);
    f.git("update-index", "--force-remove", "--", ...removed);
    const tree = f.git("write-tree");
    assert.throws(() => readLspSourceFiles(f.root, tree), {
      message: `required LSP source anchor absent from H: ${prefix}`,
    });
  }
});
