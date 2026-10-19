// Regenerate the upstream-derived corpora in tests/_fixtures/vapor-parity from a
// vuejs/core checkout at the pinned release tag:
//   node tests/tooling/support/vapor-parity/extract.ts <vuejs/core checkout>
// Then rerun the ratchet with VIZE_VAPOR_PARITY_UPDATE=reset.
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import ts from "typescript";
import { vueVersion } from "./runtime.ts";

const [core] = process.argv.slice(2);
if (!core) throw new Error("usage: extract.ts <vuejs/core checkout>");
const fixtures = new URL("../../../_fixtures/vapor-parity/", import.meta.url);
/** Sources that render differently on every run cannot be compared. */
const nondeterministic = /Math\.random|Date\b|performance\.now/;

function specs(dir: string): string[] {
  // Interop specs mount Vapor inside VDOM; their templates need both renderers.
  return readdirSync(dir)
    .filter((name) => !name.startsWith("interop"))
    .flatMap((name) => {
      const path = join(dir, name);
      if (statSync(path).isDirectory()) return specs(path);
      return path.endsWith(".spec.ts") ? [path] : [];
    });
}

function stringArgument(node: ts.CallExpression, index: number): string | undefined {
  const argument = node.arguments[index];
  return argument && (ts.isStringLiteral(argument) || ts.isNoSubstitutionTemplateLiteral(argument))
    ? argument.text
    : undefined;
}

function collect(
  dir: string,
  prefix: string,
  pick: (call: ts.CallExpression) => string | undefined,
): { id: string; source: string }[] {
  const seen = new Set<string>();
  const cases: { id: string; source: string }[] = [];
  for (const file of specs(dir).sort()) {
    const name = relative(dir, file)
      .replace(/\.spec\.ts$/, "")
      .replaceAll("/", ".");
    const sourceFile = ts.createSourceFile(
      file,
      readFileSync(file, "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );
    let index = 0;
    const visit = (node: ts.Node): void => {
      if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
        const source = pick(node);
        if (source?.trim() && !seen.has(source)) {
          seen.add(source);
          // Number before filtering so ids stay stable across filters.
          const id = `${prefix}.${name}.${index++}`;
          if (!nondeterministic.test(source)) cases.push({ id, source });
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(sourceFile);
  }
  return cases;
}

function write(file: string, provenance: string, cases: { id: string; source: string }[]): void {
  writeFileSync(new URL(file, fixtures), `${JSON.stringify({ provenance, cases }, null, 1)}\n`);
  console.log(file, cases.length);
}

write(
  "upstream-compiler-vapor.json",
  `First string argument of every compile*() call in vuejs/core v${vueVersion} packages/compiler-vapor/__tests__, deduplicated (tests/tooling/support/vapor-parity/extract.ts).`,
  collect(join(core, "packages/compiler-vapor/__tests__"), "up", (call) =>
    (call.expression as ts.Identifier).text.startsWith("compile")
      ? stringArgument(call, 0)
      : undefined,
  ),
);
write(
  "upstream-runtime-vapor-hydration.json",
  `Single-component templates passed to testHydration()/mountWithHydration() in vuejs/core v${vueVersion} packages/runtime-vapor/__tests__/hydration, deduplicated (tests/tooling/support/vapor-parity/extract.ts).`,
  collect(join(core, "packages/runtime-vapor/__tests__/hydration"), "hy", (call) => {
    if (!/^(testHydration|mountWithHydration)$/.test((call.expression as ts.Identifier).text)) {
      return undefined;
    }
    const components = call.arguments[1];
    if (components && !(ts.isIdentifier(components) && components.text === "undefined")) {
      return undefined;
    }
    const source = stringArgument(call, 0)?.trim();
    if (!source || source.includes("<script")) return undefined;
    return /^<template>([\s\S]*)<\/template>$/.exec(source)?.[1]?.trim() ?? source;
  }),
);
