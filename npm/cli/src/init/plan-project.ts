import fs from "node:fs";
import path from "node:path";

import { DEFAULT_SCRIPTS, detectJsonIndent, parsePackageJson } from "../setup/config.js";
import type { ProjectDetection } from "./detect.js";
import { skipped, type PlanDraft } from "./plan-types.js";
import type { FeatureId, FeatureSelection } from "./select.js";
import { renderTypecheckTsconfig } from "./templates.js";

/** Scripts each feature contributes, reusing the command strings `setup` ships. */
const FEATURE_SCRIPTS: Readonly<Record<FeatureId, readonly string[]>> = {
  lint: ["vize:lint"],
  bundler: [],
  fmt: ["vize:fmt", "vize:fmt:fix"],
  typecheck: ["vize:check"],
  editor: [],
};

/**
 * Plans the typecheck scaffold and the fmt/typecheck feature results.
 *
 * CLI tools use project settings and sensible defaults without a dedicated
 * config. Only typechecking needs a tsconfig scaffold; existing project and Vize
 * configs keep their exact contents.
 */
export function planToolSettings(
  detection: ProjectDetection,
  selection: FeatureSelection,
  draft: PlanDraft,
): void {
  if (selection.typecheck && detection.tsconfig === null) {
    draft.files.push({
      filename: path.join(detection.root, "tsconfig.json"),
      source: renderTypecheckTsconfig(
        detection.typescript,
        detection.framework === "nuxt" ? detection.nuxtMajor : undefined,
      ),
    });
    draft.createdFiles.push("tsconfig.json");
  }

  for (const id of ["fmt", "typecheck"] as const) {
    if (!selection[id]) {
      draft.features.push(skipped(id));
      continue;
    }
    const scaffoldsTsconfig = id === "typecheck" && detection.tsconfig === null;
    const addsScripts = FEATURE_SCRIPTS[id].some((name) => !(name in detection.scripts));
    const detail = scaffoldsTsconfig
      ? "writes tsconfig.json"
      : id === "typecheck"
        ? "uses tsconfig.json and project settings"
        : "uses project settings and formatter defaults";
    draft.features.push({
      id,
      outcome: scaffoldsTsconfig || addsScripts ? "configured" : "unchanged",
      detail:
        detection.vizeConfig === null
          ? detail
          : `${detail}; ${detection.vizeConfig} already exists and was left unchanged`,
      snippet: null,
    });
  }
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
    if (!selection[id]) {
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
    scripts[name] =
      name === "vize:check" && detection.framework === "nuxt"
        ? "nuxt prepare && vize check"
        : DEFAULT_SCRIPTS[name as keyof typeof DEFAULT_SCRIPTS];
  }
  packageJson.scripts = scripts;
  draft.files.push({
    filename: packagePath,
    source: `${JSON.stringify(packageJson, null, detectJsonIndent(source))}\n`,
  });
  draft.updatedFiles.push("package.json");
  return missing;
}
