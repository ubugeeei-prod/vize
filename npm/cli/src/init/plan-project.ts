import fs from "node:fs";
import path from "node:path";

import { DEFAULT_SCRIPTS, detectJsonIndent, parsePackageJson } from "../setup/config.js";
import type { ProjectDetection } from "./detect.js";
import { skipped, type PlanDraft } from "./plan-types.js";
import type { FeatureId, FeatureSelection } from "./select.js";
import { renderVizeConfig } from "./templates.js";

/** Scripts each feature contributes, reusing the command strings `setup` ships. */
const FEATURE_SCRIPTS: Readonly<Record<FeatureId, readonly string[]>> = {
  lint: ["vize:lint"],
  bundler: [],
  fmt: ["vize:fmt", "vize:fmt:fix"],
  typecheck: ["vize:check"],
  editor: [],
};

/**
 * Plans `vize.config.ts` and the fmt/typecheck feature results.
 *
 * Only selected features contribute a block, so asking for the formatter alone
 * does not hand the project a type checker it never opted into. An existing Vize
 * config is never rewritten: merging into a user's config is exactly the kind of
 * guess that loses their settings.
 */
export function planVizeConfig(
  detection: ProjectDetection,
  selection: FeatureSelection,
  draft: PlanDraft,
): void {
  for (const id of ["fmt", "typecheck"] as const) {
    if (!selection[id]) {
      draft.features.push(
        id === "typecheck" && detection.tsconfig === null
          ? skipped(id, "no tsconfig.json, so vize check has nothing to check")
          : skipped(id),
      );
      continue;
    }
    if (id === "typecheck" && detection.tsconfig === null) {
      draft.features.push({
        id,
        outcome: "blocked",
        detail: "vize check needs a tsconfig.json; none was found",
        snippet: null,
      });
      continue;
    }
    draft.features.push({
      id,
      outcome: detection.vizeConfig === null ? "configured" : "unchanged",
      detail:
        detection.vizeConfig === null
          ? "writes vize.config.ts"
          : `${detection.vizeConfig} already exists and was left unchanged`,
      snippet: null,
    });
  }

  const needsConfig = selection.lint || selection.fmt || selection.typecheck;
  if (!needsConfig || detection.vizeConfig !== null) {
    return;
  }
  draft.files.push({
    filename: path.join(detection.root, "vize.config.ts"),
    source: renderVizeConfig({
      lint: selection.lint,
      fmt: selection.fmt,
      typecheck: selection.typecheck && detection.tsconfig !== null,
      vite: detection.framework === "vite",
    }),
  });
  draft.createdFiles.push("vize.config.ts");
}

/**
 * Adds the scripts the selected features need.
 *
 * A script the project already defines is left alone, whatever its value: the
 * user's version of `vize:lint` outranks the default, and rewriting it would
 * make a second `init` run destructive.
 */
export function planScripts(
  detection: ProjectDetection,
  selection: FeatureSelection,
  draft: PlanDraft,
): readonly string[] {
  const wanted: string[] = [];
  for (const id of ["lint", "fmt", "typecheck"] as const) {
    if (!selection[id] || (id === "typecheck" && detection.tsconfig === null)) {
      continue;
    }
    wanted.push(...FEATURE_SCRIPTS[id]);
  }
  const missing = wanted.filter((name) => !(name in detection.scripts));
  if (missing.length === 0) {
    return [];
  }
  const packagePath = path.join(detection.root, "package.json");
  const source = fs.readFileSync(packagePath, "utf8");
  const packageJson = parsePackageJson(packagePath, source);
  const scripts = { ...detection.scripts } as Record<string, string>;
  for (const name of missing) {
    scripts[name] = DEFAULT_SCRIPTS[name as keyof typeof DEFAULT_SCRIPTS];
  }
  packageJson.scripts = scripts;
  draft.files.push({
    filename: packagePath,
    source: `${JSON.stringify(packageJson, null, detectJsonIndent(source))}\n`,
  });
  draft.updatedFiles.push("package.json");
  return missing;
}
