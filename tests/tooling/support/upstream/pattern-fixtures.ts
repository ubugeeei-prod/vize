import assert from "node:assert/strict";
import fs from "node:fs";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import vm from "node:vm";
import { patternRoot } from "./vue-language-tools.ts";

type CoverageCase = [name: string, type: string, arms: string[], exhaustive: boolean];

/** Execute only the pinned reference's fixture construction, retaining SFC imports verbatim. */
export function patternFixtures(): { cases: CoverageCase[]; sources: Map<string, string> } {
  const source = fs.readFileSync(
    path.join(patternRoot, "upstream/language-tools/packages/tsc/tests/patternedTemplates.spec.ts"),
    "utf8",
  );
  const start = source.indexOf("const cases:");
  const end = source.indexOf("const options: ts.CompilerOptions");
  assert.ok(start >= 0 && end > start, "reference fixture-construction boundaries changed");
  const construction = stripTypeScriptTypes(source.slice(start, end));
  const result = vm.runInNewContext(
    `${construction}\n({ cases, sources });`,
    {
      path: path.posix,
      __dirname: "/reference/packages/tsc/tests",
    },
    { timeout: 1_000 },
  ) as { cases: CoverageCase[]; sources: Map<string, string> };
  return {
    cases: JSON.parse(JSON.stringify(result.cases)) as CoverageCase[],
    sources: new Map([...result.sources].map(([file, text]) => [path.posix.basename(file), text])),
  };
}

export function writePatternProject(directory: string, sources: Map<string, string>): void {
  const fixtureDirectory = path.join(directory, "patterned-templates");
  fs.mkdirSync(fixtureDirectory);
  fs.mkdirSync(path.join(directory, "tsc"));
  fs.copyFileSync(path.join(patternRoot, "shared.d.ts"), path.join(directory, "tsc/shared.d.ts"));
  for (const [file, source] of sources) {
    // The reference predates Vue's click: PointerEvent update. Keep the original
    // fixture pinned; evaluate the two event-shadowing assertions against the
    // installed Vue public contract instead of an obsolete DOM event type.
    let text = source;
    if (file === "event-scopes.vue") {
      assert.equal(source.match(/as MouseEvent/g)?.length, 2);
      text = source.replaceAll(
        "as MouseEvent",
        "as Parameters<NonNullable<import('vue').HTMLAttributes['onClick']>>[0]",
      );
    }
    fs.writeFileSync(path.join(fixtureDirectory, file), text);
  }
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        allowJs: true,
        checkJs: true,
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        jsx: "preserve",
        types: [],
      },
      include: ["patterned-templates/**/*", "tsc/**/*"],
    }),
  );
  fs.writeFileSync(
    path.join(directory, "vize.config.json"),
    JSON.stringify({
      experimentals: { patternedTemplate: true },
    }),
  );
}
