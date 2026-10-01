/** Host JSON ownership at the three actual export callers only. */
export const profileHostImports: Record<string, string> = {
  "crates/vize/src/commands/profile_export.rs":
    "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};",
  "crates/vize_curator/src/inspector/stages/profile.rs":
    "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};",
  "crates/vize_atelier_sfc/examples/davinci_production_perf/attribution.rs":
    "use vize_carton::profile_export::{ProfileExportOptions, export_report};",
};

export function withoutProfileHostImports(source: string, file: string): string {
  const declaration = profileHostImports[file];
  if (!declaration) return source;
  const escaped = declaration.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return source.replace(new RegExp(`^\\s*${escaped}$`, "gmu"), "host_profile_export");
}
