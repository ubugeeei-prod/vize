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
    const args = ["check", ...project.vueGlobs, "--format", "json", "--no-config"];
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
