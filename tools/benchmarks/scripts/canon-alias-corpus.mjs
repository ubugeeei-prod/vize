/** Authored #3984 inputs and independent complete diagnostic expectations. */
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { corpusManifest } from "./type-snapshot-cli-corpus.mjs";

export const fixtureRoot = fileURLToPath(
  new URL(
    "../../../tests/_fixtures/differential/typecheck/path-alias-precedence-3984/",
    import.meta.url,
  ),
);
export const cases = JSON.parse(readFileSync(join(fixtureRoot, "cases.json"), "utf8")).cases;
const source = (name) => readFileSync(join(fixtureRoot, `${name}.txt`), "utf8");

export function prepareAliasCase(directory, item, kind, vuePackageDir) {
  const root = join(directory, item.id, kind);
  mkdirSync(join(root, "node_modules"), { recursive: true });
  symlinkSync(vuePackageDir, join(root, "node_modules", "vue"), "dir");
  symlinkSync(join(dirname(vuePackageDir), "@vue"), join(root, "node_modules", "@vue"), "dir");
  const extension = kind === "tsx" ? "tsx" : "ts";
  const paths = Object.fromEntries(
    Object.entries(item.paths).map(([key, targets]) => [
      key,
      targets.map((target) => target.replace(/\.tsx$/u, `.${extension}`)),
    ]),
  );
  const app = kind === "tsx" ? "App.tsx" : "App.vue";
  writeFileSync(join(root, app), source(app));
  for (const name of ["exact", "wildcard", "wildcardx", "genericx"])
    writeFileSync(
      join(root, `${name}.${extension}`),
      source(`${name === "exact" ? item.exact : item.wildcard}.ts`),
    );
  writeFileSync(
    join(root, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          jsx: "preserve",
          noEmit: true,
          paths,
        },
        files: [app],
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(join(root, "vize.config.json"), '{"typeChecker":{"jsxTypecheck":true}}\n');
  return { root, app, manifest: corpusManifest(root) };
}

function diagnostic(code, kind) {
  assert([2322, 2307].includes(code), "unknown authored expectation");
  return code === 2322
    ? {
        code: "TS2322",
        column: 7,
        line: kind === "tsx" ? 2 : 3,
        message: "Type 'number' is not assignable to type 'string'.",
        severity: "error",
      }
    : {
        code: "TS2307",
        column: 23,
        line: kind === "tsx" ? 1 : 2,
        message: "Cannot find module '@x' or its corresponding type declarations.",
        severity: "error",
      };
}

export function expectedServerPacket(code) {
  return {
    diagnostics: code == null ? [] : [diagnostic(code, "server")],
    errorCount: code == null ? 0 : 1,
  };
}

export function expectedCliDiagnostics(code) {
  if (code == null) return [];
  const expected = diagnostic(code, "tsx");
  return [
    ["App.tsx", `error:${expected.line}:${expected.column} [${expected.code}] ${expected.message}`],
  ];
}

export function expectedCliReport(item, root, code) {
  const compilerOptions = JSON.parse(
    readFileSync(join(root, "tsconfig.json"), "utf8"),
  ).compilerOptions;
  return {
    files: item.cliFiles.map((file) => ({
      file,
      diagnostics:
        file === "App.tsx" ? expectedCliDiagnostics(code).map(([, message]) => message) : [],
    })),
    programs: [
      {
        root: ".",
        tsconfig: "tsconfig.json",
        compilerOptions: {
          ...compilerOptions,
          paths: Object.fromEntries(
            Object.entries(item.paths).map(([pattern, targets]) => [
              pattern,
              targets.map((target) => resolve(root, target)),
            ]),
          ),
        },
        files: ["App.tsx"],
      },
    ],
    errorCount: code == null ? 0 : 1,
    warningCount: 0,
    fileCount: item.cliFiles.length,
  };
}

export function expectedOriginalDiagnostics(code) {
  if (code == null) return [];
  const expected = diagnostic(code, "tsx");
  return [
    [
      "App.tsx",
      "error",
      String(expected.line),
      String(expected.column),
      expected.code,
      expected.message,
    ],
  ];
}
