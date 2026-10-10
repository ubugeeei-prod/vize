import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { pathHostCallers } from "../../tools/support/compat/levels/path-host-callers.ts";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";

const read = (file: string) =>
  readFileSync(fileURLToPath(new URL(`../../${file}`, import.meta.url)), "utf8");
const folderHost = "crates/vize_maestro/src/server/state/workspace_folders.rs";
const pathHost = "crates/vize_maestro/src/ide/corsa_support/rename_scope.rs";
const consumers = [
  "crates/vize_maestro/src/server/state/project_contexts.rs",
  "crates/vize_maestro/src/server/state/project_contexts/paths.rs",
  "crates/vize_maestro/src/server/state/workspace_folders/config.rs",
];
const forbidden = /\bvize_carton::|use vize_carton\b/u;

test("folder snapshots admit only the exact successor at the existing physical host", () => {
  const historical =
    "vize_carton::config::load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source(Some(&root))";
  const current = "vize_carton::config::load_project_config_with_source(Some(&root))";
  assert.equal(read(folderHost).split(current).length - 1, 1);
  for (const file of [folderHost, folderHost.replaceAll("/", "\\")]) {
    for (const call of [historical, current])
      assert.doesNotMatch(withoutHostRuntimeReferences(call, file), forbidden);
    for (const call of [
      "vize_carton::config::load_project_config_with_source_extra(None)",
      "vize_carton::config::load_project_config_with_sourceα(None)",
      "use vize_carton::config::LoadedProjectConfig;",
      "use vize_carton::config::*;",
      "use vize_carton::{config::load_project_config_with_source};",
    ])
      assert.match(withoutHostRuntimeReferences(call, file), forbidden);
  }
  for (const file of [...consumers, "davinci/vize_l1/src/config.rs"])
    assert.match(withoutHostRuntimeReferences(current, file), forbidden);
});

test("stage consumers receive the existing document and path values without host imports", () => {
  for (const file of consumers) assert.doesNotMatch(read(file), forbidden, file);
  const config = read("crates/vize_maestro/src/server/state/config.rs");
  assert.match(
    config,
    /install_project_linter_context\(\s*dir,\s*&loaded\.project\.document,\s*loaded\.project\.source_path\.as_deref\(\),\s*loaded\.project\.project_root\.as_deref\(\),\s*\)/u,
  );
});

test("path normalization retains the exact finite owner and rejects surplus host calls", () => {
  assert.match(
    read("crates/vize_maestro/src/ide.rs"),
    /#\[cfg\(feature = "native"\)\]\s*pub\(crate\) use corsa_support::normalize_physical_path;/u,
  );
  assert.match(
    read(consumers[1]),
    /crate::ide::normalize_physical_path\(physical\.join\(relative\)\)/u,
  );
  assert.doesNotMatch(read(consumers[1]), /crate::ide::corsa_support::normalize_physical_path/u);
  assert.deepEqual(pathHostCallers[pathHost], [0, 0, 1]);
  const source = read(pathHost);
  assert.doesNotMatch(withoutHostRuntimeReferences(source, pathHost), forbidden);
  const call = "vize_carton::path::normalize_windows_verbatim_path(path)";
  assert.match(withoutHostRuntimeReferences(`${source}\n${call}`, pathHost), forbidden);
  for (const file of consumers) assert.match(withoutHostRuntimeReferences(call, file), forbidden);
});
