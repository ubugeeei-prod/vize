import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";

const hostFiles = [
  "crates/vize/src/config.rs",
  "crates/vize_maestro/src/server/state/config.rs",
  "crates/vize_maestro/src/server/state/workspace_folders.rs",
  "crates/vize_maestro/src/server/state/batch_cache.rs",
  "crates/vize_maestro/src/server/state/corsa.rs",
  "crates/vize/src/lint_plan/matcher.rs",
];
const forbiddenCartonStorage = /\bvize_carton::|use vize_carton\b/u;
const source = (path: string) =>
  readFileSync(fileURLToPath(new URL(`../../${path}`, import.meta.url)), "utf8");

test("all Rust callers retire the old L0 matcher path", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const files = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of files) {
    assert.doesNotMatch(source(file), /\bvize_l0\s*::\s*config\s*::\s*matcher\b/u, file);
  }
  assert.match(
    source("crates/vize/src/commands/lint/collect.rs"),
    /crate::lint_plan::matcher::absolute_path\(config_dir, &cwd\)/u,
  );
});

test("actual config hosts use only the reviewed Carton facade and loader calls", () => {
  for (const file of hostFiles) {
    assert.doesNotMatch(withoutHostRuntimeReferences(source(file), file), forbiddenCartonStorage);
    assert.doesNotMatch(
      withoutHostRuntimeReferences(source(file), file.replaceAll("/", "\\")),
      forbiddenCartonStorage,
    );
  }
});

test("host config allowance keeps Carton storage and effective type imports forbidden", () => {
  for (const file of hostFiles) {
    for (const importSource of [
      "use vize_carton::String;",
      "use vize_carton::{String, config::load_config_with_source};",
      "use vize_carton::config::TypeCheckerConfig;",
    ]) {
      assert.match(
        withoutHostRuntimeReferences(`${source(file)}\n${importSource}`, file),
        forbiddenCartonStorage,
      );
    }
  }
});

test("the config facade is allowed only at the existing CLI compatibility facade", () => {
  const facade = "pub use vize_carton::config::*;";
  assert.match(withoutHostRuntimeReferences(facade, hostFiles[1]), forbiddenCartonStorage);
  assert.match(
    withoutHostRuntimeReferences(facade, "davinci/vize_l1/src/dialect/vue.rs"),
    forbiddenCartonStorage,
  );
});

test("unreviewed config functions and similar function names remain forbidden", () => {
  for (const call of [
    "vize_carton::config::load_lib_config_with_source(None)",
    "vize_carton::config::load_config_with_source_storage(None)",
  ]) {
    assert.match(withoutHostRuntimeReferences(call, hostFiles[1]), forbiddenCartonStorage);
  }
  assert.match(
    withoutHostRuntimeReferences(
      "vize_carton::config::load_lsp_config_snapshot(None)",
      hostFiles[2],
    ),
    forbiddenCartonStorage,
  );
});

test("project selection is allowed only at the two existing host callers", () => {
  const call = "vize_carton::config::ProjectModel::new(None, None, &config)";
  for (const file of hostFiles.slice(0, 3)) {
    assert.match(withoutHostRuntimeReferences(call, file), forbiddenCartonStorage);
  }
  assert.match(
    withoutHostRuntimeReferences(call, "davinci/vize_l1/src/config.rs"),
    forbiddenCartonStorage,
  );
  for (const file of hostFiles.slice(3, 5)) {
    for (const unreviewed of [
      "use vize_carton::config::ProjectModel;",
      "vize_carton::config::ProjectModel::new_storage(None)",
      "vize_carton::config::ProjectModel::default()",
    ]) {
      assert.match(withoutHostRuntimeReferences(unreviewed, file), forbiddenCartonStorage);
    }
  }
});

test("matcher imports allow only the existing scoped host symbols", () => {
  for (const file of hostFiles) {
    for (const unreviewed of [
      "use vize_carton::config::matcher::GlobSequence;",
      "use vize_carton::config::matcher::String;",
      "use vize_carton::config::matcher::{LintPlanScope, String};",
      "pub(crate) use vize_carton::config::matcher::{LintPlanScope, absolute_path, normalize_path, String};",
    ]) {
      assert.match(withoutHostRuntimeReferences(unreviewed, file), forbiddenCartonStorage);
    }
  }
  for (const file of hostFiles.filter((file) => !file.endsWith("workspace_folders.rs"))) {
    assert.match(
      withoutHostRuntimeReferences("use vize_carton::config::matcher::LintPlanScope;", file),
      forbiddenCartonStorage,
    );
  }
  assert.match(
    withoutHostRuntimeReferences(
      "pub(crate) use vize_carton::config::matcher::GlobSequence;",
      "davinci/vize_l1/src/config.rs",
    ),
    forbiddenCartonStorage,
  );
});
