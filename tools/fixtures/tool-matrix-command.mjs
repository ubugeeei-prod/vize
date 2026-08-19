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
    // The typechecker is the one tool pinned to a single `--tsconfig`, so its
    // corpus has to be the files that config owns. Passing `vueGlobs` — which
    // the other tools share and which may span sibling projects — asks one
    // config to answer for files it never included, and the vue-tsc baseline,
    // built from the same list with the same options, then cannot resolve the
    // aliases those files rely on (#4454).
    const globs = project.typecheckPerformance?.corpusGlobs ?? project.vueGlobs;
    const args = ["check", ...globs, "--format", "json", "--no-config"];
    if (project.tsconfig != null) args.push("--tsconfig", project.tsconfig);
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
