import { withoutProfileHostImports } from "./davinci-profile-host-imports.ts";
import { withoutI18nHostImports } from "./davinci-i18n-host-imports.ts";
import { withoutSourceIoHostReferences } from "./davinci-source-io-host-imports.ts";
import { withoutPathHostReferences } from "./davinci-path-host-imports.ts";
import { withoutTimingHostImport } from "./davinci-timing-host-imports.ts";

/** Remove only reviewed legacy host APIs before checking L0 storage imports. */
export function withoutHostRuntimeReferences(source: string, relativePath: string): string {
  const file = relativePath.replaceAll("\\", "/");
  let storage = source.replace(
    /\bvize_carton::(?:corsa_(?:api_mode|resolver)|VERSION)\b/gu,
    "host_runtime",
  );
  // Nuxt project facts come from the filesystem. The CLI and the editor share
  // the one Carton reader so both lint a Nuxt project the same way.
  storage = storage.replace(/\bvize_carton::nuxt\b/gu, "host_nuxt_project");
  if (file === "crates/vize/src/config.rs") {
    storage = storage.replace(/^pub use vize_carton::config::\*;$/gmu, "host_config_facade");
  }
  if (file === "crates/vize_maestro/src/server/state/config/host.rs") {
    // Snapshot identity and filesystem ownership stay in Carton. Only this
    // private facade can expose the reviewed host APIs to state consumers.
    storage = storage.replace(
      /^pub\(in crate::server::state\) use vize_carton::config::(?:ConfigEvaluationSession|ConfigLoadFailureIdentity|LoadedLspConfig|ProjectModel|load_config_with_source|load_lsp_config_snapshot|load_lsp_config_snapshot_observed|load_project_config_with_source);$/gmu,
      "host_config_snapshot_facade",
    );
    storage = storage.replace(
      /^pub\(in crate::server::state\) use vize_carton::config::matcher::CompilerWhitespacePlan;$/gmu,
      "host_config_matcher_facade",
    );
    storage = storage.replace(
      /^pub\(in crate::server::state\) use vize_carton::config::formatter_lint_plan::FormatterLinterPlan;$/gmu,
      "host_formatter_lint_plan_facade",
    );
  }
  if (file === "crates/vize_maestro/src/server/state/config.rs") {
    storage = storage.replace(
      /\bvize_carton::config::\s*(?:load_lsp_config_snapshot|load_config_with_source)\b(?=\s*\()/gu,
      "host_config_loader",
    );
  }
  if (file === "crates/vize_maestro/src/server/state/workspace_folders.rs") {
    storage = storage.replace(
      /\bvize_carton::config::\s*load_config_and_linter_plan_with_config_rule_options_and_lint_features_and_source\b(?=\s*\()/gu,
      "host_config_loader",
    );
    storage = storage.replace(
      /(?<![\p{ID_Continue}])vize_carton::config::\s*load_project_config_with_source(?![\p{ID_Continue}])(?=\s*\()/gu,
      "host_config_loader",
    );
    storage = storage.replace(
      /^use vize_carton::config::matcher::(?:LintPlanScope|\{LintPlanScope, ProjectIgnoreSet\});$/gmu,
      "host_config_matcher",
    );
  }
  if (
    file === "crates/vize_patina/src/plugin_batch/lint.rs" ||
    file === "crates/vize_patina/src/plugin_batch/lint/rules.rs"
  ) {
    // Plugin lint jobs read the plan Carton verified; they own no config storage.
    storage = storage.replace(
      /^use vize_carton::config::plugin_plan::\{(?:PlannedCapability, PlannedPlugin(?:, VerifiedPluginPlan)?)\};$/gmu,
      "host_plugin_plan",
    );
  }
  if (file === "crates/vize_patina/src/plugin_batch/lint/host.rs") {
    // The per-document host drives Carton's plugin sidecar for the plan Carton verified.
    storage = storage.replace(
      /^use vize_carton::config::plugin_plan::\{VerifiedPluginPlan, document_plugin_plan\};$/gmu,
      "host_plugin_plan",
    );
    storage = storage.replace(
      /^use vize_carton::config::\{PluginLintSidecar, SidecarBudget, SidecarReply\};$/gmu,
      "host_plugin_sidecar",
    );
  }
  if (file === "crates/vize_vitrine/src/plugin_plans.rs") {
    // In-process compiler hosts keep the plans Carton verifies (#13074).
    storage = storage.replace(
      /^\s*use vize_carton::config::plugin_plan::(?:\{PluginPlanError, VerifiedPluginPlan, verify_plugin_plan\}|verify_plugin_plan);$/gmu,
      "host_plugin_plan",
    );
  }
  if (file === "crates/vize_vitrine/src/napi/plugin_lint.rs") {
    // Oxlint's plugin session loads its config through the same checked Carton loader.
    storage = storage.replace(
      /\bvize_carton::config::try_load_project_config_with_source\b(?=\()/gu,
      "host_config_loader",
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
