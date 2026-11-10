import { npmTagForVersion } from "../../../support/release/npm-tag.ts";

const [version] = process.argv.slice(2);
if (version === undefined) {
  console.log("Usage: node tools/commands/release/npm/tag.ts <version>");
  process.exitCode = 1;
} else {
  console.log(npmTagForVersion(version));
}
