import { renamedWorkflowStems } from "./workflow-renames.ts";

/** Map renamed workflow files and their program directories back to the stems frozen records name. */
export function restoreWorkflowNames(text: string): string {
  let restored = text;
  for (const [stem, recorded] of renamedWorkflowStems)
    restored = restored
      .replace(
        new RegExp(`tools/ci/workflows/${stem}/scripts(?=[/"\\s]|$)`, "gmu"),
        `.github/scripts/workflows/${recorded}`,
      )
      .replaceAll(`tools/ci/workflows/${stem}/jobs/`, `tools/config/pkl/parts/${recorded}/`)
      .replace(new RegExp(`(?<![\\w-])${stem}\\.yml`, "gu"), `${recorded}.yml`);
  return restored;
}

/** Map the tools/ci layout back to the paths frozen workflow records name. */
export function restoreCiLibraryPaths(text: string): string {
  return restoreWorkflowNames(text)
    .replaceAll("../../../../tools/ci/lib/scripts/", "../shared/")
    .replaceAll("tools/ci/commands/", "tools/commands/ci/")
    .replaceAll("tools/ci/lib/scripts/", ".github/scripts/workflows/shared/");
}

/** Apply restoreCiLibraryPaths to every string inside a parsed workflow value. */
export function restoreCiLibraryValue<T>(value: T): T {
  return JSON.parse(restoreCiLibraryPaths(JSON.stringify(value))) as T;
}
