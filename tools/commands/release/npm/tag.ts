function npmTagForVersion(version: string): string {
  if (version.includes("-alpha")) return "alpha";
  if (version.includes("-beta")) return "beta";
  if (version.includes("-rc")) return "rc";
  return "latest";
}

const [version] = process.argv.slice(2);
if (version === undefined) {
  console.log("Usage: node tools/commands/release/npm/tag.ts <version>");
  process.exitCode = 1;
} else {
  console.log(npmTagForVersion(version));
}
