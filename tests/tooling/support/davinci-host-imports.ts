import { withoutProfileHostImports } from "./davinci-profile-host-imports.ts";
import { withoutI18nHostImports } from "./davinci-i18n-host-imports.ts";
import { withoutSourceIoHostReferences } from "./davinci-source-io-host-imports.ts";
import { withoutPathHostReferences } from "./davinci-path-host-imports.ts";
import { withoutTimingHostImport } from "./davinci-timing-host-imports.ts";

/** Remove only reviewed legacy host APIs before checking L0 storage imports. */
export function withoutHostRuntimeReferences(source: string, relativePath: string): string {
  const file = relativePath.replaceAll("\\", "/");
  let storage = source.replace(/\bvize_carton::corsa_(?:api_mode|resolver)\b/gu, "host_runtime");
  if (file === "crates/vize/src/config.rs") {
    storage = storage.replace(/^pub use vize_carton::config::\*;$/gmu, "host_config_facade");
  }
  if (file === "crates/vize_maestro/src/server/state/config.rs") {
    storage = storage.replace(
      /\bvize_carton::config::\s*(?:load_lsp_config_snapshot|load_config_with_source)\b/gu,
      "host_config_loader",
    );
  }
  if (file === "crates/vize_maestro/src/server/state/workspace_folders.rs") {
    storage = storage.replace(
      /\bvize_carton::config::\s*load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source\b/gu,
      "host_config_loader",
    );
    storage = storage.replace(
      /^use vize_carton::config::matcher::(?:LintPlanScope|\{LintPlanScope, ProjectIgnoreSet\});$/gmu,
      "host_config_matcher",
    );
  }
  if (file === "crates/vize/src/lint_plan/matcher.rs") {
    storage = storage.replace(
      /^pub\(crate\) use vize_carton::config::matcher::(?:GlobSequence|\{LintPlanScope, absolute_path, normalize_path\});$/gmu,
      "host_config_matcher",
    );
  }
  if (
    file === "crates/vize_maestro/src/server/state/batch_cache.rs" ||
    file === "crates/vize_maestro/src/server/state/corsa.rs"
  ) {
    storage = storage.replace(
      /\bvize_carton::config::ProjectModel::new\b/gu,
      "host_project_selection",
    );
  }
  if (file === "crates/vize_maestro/src/server/state/module_links.rs") {
    storage = storage.replace(
      /^use vize_carton::config::ProjectModel;$/gmu,
      "host_applied_project_selection",
    );
  }
  storage = withoutPathHostReferences(storage, file);
  storage = withoutTimingHostImport(storage, file);
  storage = withoutSourceIoHostReferences(storage, file);
  return withoutProfileHostImports(withoutI18nHostImports(storage, file), file);
}
