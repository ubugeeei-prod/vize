/** Host JSON ownership at the three actual export callers only. */
export const profileHostImports: Record<string, string> = {
  "crates/vize/src/commands/profile_export.rs":
    "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};",
  "crates/vize_curator/src/inspector/stages/profile.rs": `use vize_carton::profile_export::{
    ProfileExportBudget, ProfileExportOptions, export_report_from_snapshots,
};`,
  "crates/vize_atelier_sfc/examples/davinci_production_perf/attribution.rs":
    "use vize_carton::profile_export::{ProfileExportOptions, export_report};",
};

// Preserve the exact preceding source-compatible caller declaration only at
// its actual Curator path; it grants no other host or storage API.
export const previousCuratorProfileImport =
  "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};";

export function withoutProfileHostImports(source: string, file: string): string {
  const declaration = profileHostImports[file];
  if (!declaration) return source;
  const declarations =
    file === "crates/vize_curator/src/inspector/stages/profile.rs"
      ? [declaration, previousCuratorProfileImport]
      : [declaration];
  return declarations.reduce((remaining, admitted) => {
    const escaped = admitted.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    return remaining.replace(new RegExp(`^\\s*${escaped}$`, "gmu"), "host_profile_export");
  }, source);
}
