import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { isolatedTsconfigOverlayPath } from "./typecheck-baseline-outside-paths.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function typecheckCorpusGlobs(project) {
  return project.typecheckPerformance?.corpusGlobs ?? project.vueGlobs;
}

export function typecheckTsconfigPath(project) {
  if (typeof project.tsconfig !== "string") return null;
  const overlayRel = isolatedTsconfigOverlayPath(project.tsconfig);
  const overlayAbs =
    typeof project.fixturePath === "string"
      ? resolve(repoRoot, project.fixturePath, overlayRel)
      : resolve(overlayRel);
  return existsSync(overlayAbs) ? overlayRel : project.tsconfig;
}

export function toolArgs(project, tool, compilerOutputDir) {
  if (tool === "compiler") {
    return [
      "build",
      ...project.vueGlobs,
      "--format",
      "json",
      "--output",
      compilerOutputDir,
      "--template-syntax",
      "quirks",
      "--continue-on-error",
      "--no-config",
    ];
  }
  if (tool === "linter") {
    return [
      "lint",
      ...project.vueGlobs,
      "--format",
      "json",
      "--preset",
      "ecosystem",
      "--no-config",
    ];
  }
  if (tool === "typechecker") {
    // One `--tsconfig` cannot answer for sibling apps that `vueGlobs` still
    // cover for compiler/linter/formatter. Narrow the typecheck corpus to the
    // files that config owns, or vue-tsc's baseline inherits the same
    // unresolvable aliases and the comparison reports fake FNs (#4454).
    const args = ["check", ...typecheckCorpusGlobs(project), "--format", "json", "--no-config"];
    const tsconfig = typecheckTsconfigPath(project);
    if (tsconfig != null) args.push("--tsconfig", tsconfig);
    return args;
  }
  return ["fmt", ...project.vueGlobs, "--check", "--no-config"];
}

export function displayCommand(command, args) {
  return [command, ...args].map(shellQuote).join(" ");
}

function shellQuote(value) {
  return /^[A-Za-z0-9_./:=@*-]+$/.test(value) ? value : `'${value.replaceAll("'", "'\\''")}'`;
}
