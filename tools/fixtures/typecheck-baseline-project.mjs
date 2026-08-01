import { writeFileSync } from "node:fs";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";

/**
 * Materialize the exact Vue corpus Vize checked as a vue-tsc project.
 *
 * The fixture's pinned tsconfig remains the source of compiler options, while
 * `files` replaces solution-style or incomplete include lists. This makes the
 * baseline compare the same authored SFCs instead of silently checking none.
 */
export function materializeBaselineProject(fixtureRoot, reportDir, project, vizeReport) {
  const outputPath = join(reportDir, `${project.id}-vue-tsc.tsconfig.json`);
  const configDir = dirname(outputPath);
  const sourceProject = project.typecheckPerformance?.baseline?.tsconfig ?? project.tsconfig;
  const config = {
    extends: configRelativePath(configDir, resolve(fixtureRoot, sourceProject)),
    files: vizeReport.files
      .slice(0, vizeReport.fileCount)
      .map((entry) => configRelativePath(configDir, resolve(fixtureRoot, entry.file))),
    include: [],
    references: [],
  };
  const source = `${JSON.stringify(config, null, 2)}\n`;
  writeFileSync(outputPath, source);
  return { path: outputPath, source, sourceProject };
}

function configRelativePath(from, to) {
  const path = relative(from, to).replaceAll("\\", "/");
  if (isAbsolute(path) || /^[A-Za-z]:\//u.test(path)) return path;
  return path.startsWith(".") ? path : `./${path}`;
}
