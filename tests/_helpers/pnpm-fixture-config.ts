import fs from "node:fs";

export function patchPnpmMinimumReleaseAgeExclude(
  workspacePath: string,
  packageName: string,
): void {
  const source = fs.readFileSync(workspacePath, "utf8");
  if (source.includes(packageName)) return;

  const entry = `  - '${packageName}'\n`;
  const nextSource = source.includes("minimumReleaseAgeExclude:\n")
    ? source.replace("minimumReleaseAgeExclude:\n", `minimumReleaseAgeExclude:\n${entry}`)
    : `${source.trimEnd()}\nminimumReleaseAgeExclude:\n${entry}`;
  fs.writeFileSync(workspacePath, nextSource);
}
