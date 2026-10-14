import fs from "node:fs";
import path from "node:path";
import { prepareOutputDir, resolveFixturePath } from "./helpers.ts";

/**
 * A project root whose `sass` and `stylus` packages are the test stand-ins in
 * fixtures/scoped-preprocessor, so preprocess-then-scope runs without the
 * real preprocessors installed.
 */
export function createFakePreprocessorRoot(name: string): string {
  const root = prepareOutputDir(name);
  const fakeScss = JSON.stringify(resolveFixturePath("scoped-preprocessor", "fake-scss.cjs"));
  const packages: Record<string, string> = {
    sass: `const { compile } = require(${fakeScss});\nexports.compileString = (source) => ({ css: compile(source) });\n`,
    // Indented `selector\n  property value` pairs only.
    stylus:
      "module.exports = (source) => ({\n" +
      "  set() { return this; },\n" +
      "  include() { return this; },\n" +
      "  render: () => source.replace(/^(\\S[^\\n]*)\\n\\s+([\\w-]+) ([^\\n]+)$/gm, '$1 { $2: $3; }'),\n" +
      "});\n",
  };
  fs.writeFileSync(path.join(root, "package.json"), "{}\n");
  for (const [pkg, source] of Object.entries(packages)) {
    const dir = path.join(root, "node_modules", pkg);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.join(dir, "package.json"), `{ "name": "${pkg}", "main": "index.cjs" }\n`);
    fs.writeFileSync(path.join(dir, "index.cjs"), source);
  }
  return root;
}
