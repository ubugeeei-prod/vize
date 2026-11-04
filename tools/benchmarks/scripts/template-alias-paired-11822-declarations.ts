import type { PackageManifest } from "./template-alias-paired-11822-types.ts";
import type {
  Diagnostic,
  SourceFile,
  TypeScriptApi,
} from "../../../tests/_helpers/template-alias-typescript-11822.ts";
export type { TypeScriptApi } from "../../../tests/_helpers/template-alias-typescript-11822.ts";
interface ParsedSourceFile extends SourceFile {
  readonly parseDiagnostics: readonly Diagnostic[];
}
export interface DeclarationFile {
  path: string;
  bytes: number;
  sha256: string;
  relativeImports: {
    specifier?: string;
    referencePath?: string;
    referenceTypes?: string;
    target: string;
  }[];
  externalImports: string[];
  externalTypeReferences: string[];
}
export interface DeclarationClosure {
  parserVersion: string;
  exports: { condition: string[]; target: string }[];
  entries: string[];
  files: DeclarationFile[];
  unreachableShippedDeclarations: string[];
}
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, lstatSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";

const declaration = /\.d\.(?:ts|mts|cts)$/;
const inside = (root: string, path: string) => {
  const suffix = relative(root, path);
  return suffix === "" || (!suffix.startsWith("..") && !isAbsolute(suffix));
};

export function declarationParser(path: string | undefined): TypeScriptApi {
  assert.ok(
    path,
    "the exact source-pinned TypeScript JS API is required for declaration admission",
  );
  const ts = createRequire(import.meta.url)(resolve(path)) as TypeScriptApi;
  assert.equal(typeof ts.resolveModuleName, "function");
  assert.equal(typeof ts.preProcessFile, "function");
  return ts;
}

export function assertPublishFiles(root: string, manifest: PackageManifest) {
  for (const name of ["package.json", ...(manifest.files ?? [])]) {
    assert.ok(
      !/[*!?{}]/.test(name) && !name.includes(".."),
      `unsupported publish file pattern ${name}`,
    );
    assert.ok(existsSync(join(root, name)), `missing declared publish path ${name}`);
  }
}

/** Physical admission, independent of skipLibCheck and whether a type is instantiated. */
export function declarationClosure(directory: string, ts: TypeScriptApi): DeclarationClosure {
  const root = realpathSync(directory);
  const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as PackageManifest;
  assertPublishFiles(root, manifest);
  const entries = new Set<string>();
  const exports: DeclarationClosure["exports"] = [];
  const collectExports = (value: unknown, path: string[] = [], typed = false): void => {
    if (typeof value === "string") {
      if (typed || declaration.test(value)) {
        assert.ok(
          declaration.test(value) &&
            !isAbsolute(value) &&
            (path[0] !== "exports" || value.startsWith("./")),
          `unsupported types export ${value}`,
        );
        exports.push({ condition: path, target: value });
        entries.add(resolve(root, value));
      }
    } else if (value && typeof value === "object") {
      for (const [key, child] of Object.entries(value))
        collectExports(child, [...path, key], typed || key === "types");
    }
  };
  if (manifest.types !== undefined) collectExports(manifest.types, ["types"], true);
  if (manifest.typings !== undefined) collectExports(manifest.typings, ["typings"], true);
  collectExports(manifest.exports, ["exports"]);
  const shippedDeclarations = new Set<string>();
  const listedDirectories = new Set<string>();
  const visitListed = (path: string, explicit = true): void => {
    assert.ok(inside(root, realpathSync(path)), "declared path escapes package");
    const actual = realpathSync(path),
      stat = lstatSync(actual);
    if (stat.isDirectory()) {
      if (listedDirectories.has(actual)) return;
      listedDirectories.add(actual);
      for (const name of readdirSync(actual)) visitListed(join(actual, name), false);
    } else if (declaration.test(actual)) {
      shippedDeclarations.add(actual);
      if (explicit) entries.add(actual);
    }
  };
  for (const name of manifest.files ?? []) visitListed(join(root, name));
  const visited = new Map<string, DeclarationFile>();
  const walk = (requested: string): void => {
    assert.ok(existsSync(requested), `missing declaration target ${relative(root, requested)}`);
    const path = realpathSync(requested);
    assert.ok(inside(root, path), `relative declaration escaped package ${path}`);
    if (visited.has(path)) return;
    assert.ok(lstatSync(path).isFile(), `declaration target is not a file ${path}`);
    const bytes = readFileSync(path),
      text = bytes.toString("utf8");
    const parsed = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true);
    assert.deepEqual(
      (parsed as ParsedSourceFile).parseDiagnostics,
      [],
      `malformed declaration ${path}`,
    );
    const receipt: DeclarationFile = {
      path: relative(root, path),
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      relativeImports: [],
      externalImports: [],
      externalTypeReferences: [],
    };
    visited.set(path, receipt);
    const info = ts.preProcessFile(text, true, true);
    for (const item of info.importedFiles) {
      const specifier = item.fileName;
      if (!specifier.startsWith(".")) {
        receipt.externalImports.push(specifier);
        continue;
      }
      const resolved = ts.resolveModuleName(
        specifier,
        path,
        {
          module: ts.ModuleKind.NodeNext,
          moduleResolution: ts.ModuleResolutionKind.NodeNext,
          allowImportingTsExtensions: true,
          allowJs: false,
        },
        ts.sys,
      ).resolvedModule;
      assert.ok(
        resolved && /\.(?:ts|mts|cts|tsx)$/.test(resolved.resolvedFileName),
        `missing relative declaration ${specifier} from ${receipt.path}`,
      );
      receipt.relativeImports.push({
        specifier,
        target: relative(root, resolved.resolvedFileName),
      });
      walk(resolved.resolvedFileName);
    }
    for (const reference of info.referencedFiles) {
      const target = resolve(dirname(path), reference.fileName);
      receipt.relativeImports.push({
        referencePath: reference.fileName,
        target: relative(root, target),
      });
      walk(target);
    }
    for (const reference of info.typeReferenceDirectives) {
      const name = reference.fileName;
      assert.ok(
        !isAbsolute(name) && !/^(?:[A-Za-z]:[\\/]|\\\\|[A-Za-z][A-Za-z\d+.-]*:)/.test(name),
        `absolute type reference is forbidden: ${name}`,
      );
      if (!name.startsWith(".")) {
        receipt.externalTypeReferences.push(name);
        continue;
      }
      const resolved = ts.resolveTypeReferenceDirective(
        name,
        path,
        {
          module: ts.ModuleKind.NodeNext,
          moduleResolution: ts.ModuleResolutionKind.NodeNext,
        },
        ts.sys,
      ).resolvedTypeReferenceDirective;
      const target = resolved?.resolvedFileName;
      assert.ok(
        typeof target === "string" && declaration.test(target),
        `missing relative type reference ${name} from ${receipt.path}`,
      );
      receipt.relativeImports.push({
        referenceTypes: name,
        target: relative(root, target),
      });
      walk(target);
    }
  };
  for (const path of [...entries].sort()) walk(path);
  return {
    parserVersion: ts.version,
    exports,
    entries: [...entries].map((path) => relative(root, path)).sort(),
    files: [...visited.values()].sort((a, b) => a.path.localeCompare(b.path)),
    unreachableShippedDeclarations: [...shippedDeclarations]
      .filter((path) => !visited.has(path))
      .map((path) => relative(root, path))
      .sort(),
  };
}

export function typecheckHarness(directory: string, apiPath: string, nodeTypesRoot: string): void {
  const ts = declarationParser(apiPath);
  const rootNames = readdirSync(directory)
    .filter((name) => name.startsWith("template-alias-paired-11822") && name.endsWith(".ts"))
    .map((name) => join(directory, name));
  const program = ts.createProgram({
    rootNames,
    options: {
      target: ts.ScriptTarget.ES2024,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      allowImportingTsExtensions: true,
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      types: ["node"],
      typeRoots: [dirname(nodeTypesRoot)],
      paths: { typescript: [apiPath.replace(/\.js$/, ".d.ts")] },
    },
  });
  const errors = ts.getPreEmitDiagnostics(program);
  if (errors.length)
    console.error(
      ts.formatDiagnosticsWithColorAndContext(errors, {
        getCurrentDirectory: () => directory,
        getCanonicalFileName: (name) => name,
        getNewLine: () => "\n",
      }),
    );
  assert.equal(errors.length, 0, "typed campaign harness failed source-pinned TypeScript noEmit");
}
