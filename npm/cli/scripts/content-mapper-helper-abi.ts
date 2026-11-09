import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";

const subpaths = [
  "./content-mapper-helpers",
  "./virtual-ts",
  "./virtual-ts-checker",
  "./virtual-ts-checker-strict",
];
const nativeReferences = "crates/vize_canon/src/virtual_ts/helpers/shared_types.rs";
const output = "crates/vize/src/commands/content_mapper/helper-abi.json";

/** Qualify the native consumer's complete SDK declaration closure at generation time. */
export function contentMapperHelperAbi(root: string, pending: ReadonlyMap<string, string>) {
  const read = (file: string) =>
    pending.get(path.relative(root, file).split(path.sep).join("/")) ?? readFileSync(file, "utf8");
  const packageRoot = path.join(root, "npm/cli");
  const manifest = JSON.parse(read(path.join(packageRoot, "package.json")));
  const references = read(path.join(root, nativeReferences));
  const consumers = [...references.matchAll(/=>\s*\("[^"\n]+",\s*"([^"\n]+)"\)/gu)]
    .map((match) => match[1]!)
    .sort();
  assert.ok(consumers.length > 0, "native shared_type_refs! consumer list is empty");
  assert.equal(new Set(consumers).size, consumers.length, "duplicate native helper exports");
  const coreImports = ts.createSourceFile(
    "native-core-imports.d.ts",
    read(
      path.join(
        root,
        "crates/vize_canon/src/virtual_ts/helpers/declarations/content-mapper-type-imports.d.ts.txt",
      ),
    ),
    ts.ScriptTarget.Latest,
    true,
  );
  const coreConsumers = coreImports.statements.flatMap((statement) => {
    if (!ts.isImportDeclaration(statement)) return [];
    const bindings = statement.importClause?.namedBindings;
    assert.ok(bindings && ts.isNamedImports(bindings), "native core helper imports must be named");
    return bindings.elements.map((element) => (element.propertyName ?? element.name).text);
  });
  assert.ok(coreConsumers.length > 0, "native core consumer list is empty");
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    skipLibCheck: false,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.Preserve,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    types: [],
  };
  const host = ts.createCompilerHost(options);
  const originalRead = host.readFile.bind(host);
  host.readFile = (file) => {
    const relative = path.relative(root, file).split(path.sep).join("/");
    return pending.get(relative) ?? originalRead(file);
  };
  const originalExists = host.fileExists.bind(host);
  host.fileExists = (file) =>
    pending.has(path.relative(root, file).split(path.sep).join("/")) || originalExists(file);

  function targets(value: unknown): string[] {
    if (typeof value === "string") {
      assert.match(value, /^\.\/[\w./-]+\.d\.(?:ts|mts|cts)$/u, "unknown SDK type target");
      return [value];
    }
    assert.ok(value && typeof value === "object" && !Array.isArray(value), "SDK type condition");
    return [...new Set(Object.values(value).flatMap(targets))];
  }
  const modules = subpaths.map((subpath) => {
    const exports = manifest.exports[subpath];
    const entries = targets(exports).map((target) => path.resolve(packageRoot, target));
    const program = ts.createProgram(entries, options, host);
    const checker = program.getTypeChecker();
    const closure = new Set<string>();
    function visit(file: string) {
      if (closure.has(file)) return;
      const relative = path.relative(packageRoot, file);
      assert.ok(
        !path.isAbsolute(relative) && relative !== ".." && !relative.startsWith(`..${path.sep}`),
        "SDK relative import escaped package",
      );
      closure.add(file);
      const source = program.getSourceFile(file);
      assert.ok(source, `missing SDK declaration ${file}`);
      assert.equal(program.getSyntacticDiagnostics(source).length, 0, `SDK syntax: ${file}`);
      const dependencies = ts.preProcessFile(source.text, true, true);
      for (const referenced of dependencies.referencedFiles)
        visit(path.resolve(path.dirname(file), referenced.fileName));
      for (const referenced of dependencies.typeReferenceDirectives) {
        const dependency = ts.resolveTypeReferenceDirective(
          referenced.fileName,
          file,
          options,
          host,
        ).resolvedTypeReferenceDirective?.resolvedFileName;
        assert.ok(dependency, `missing SDK type dependency ${file}: ${referenced.fileName}`);
        visit(dependency);
      }
      for (const imported of dependencies.importedFiles) {
        if (!imported.fileName.startsWith(".")) {
          assert.equal(imported.fileName, "vue", "only Vue has an external SDK authority gate");
          continue;
        }
        const dependency = ts.resolveModuleName(
          imported.fileName,
          file,
          options,
          host,
        ).resolvedModule;
        assert.ok(dependency, `missing SDK dependency ${file}: ${imported.fileName}`);
        visit(dependency.resolvedFileName);
      }
      // Unlike skipLibCheck, inspect private bindings and their complete types
      // as well as the public surface. An unchecked declaration is not an ABI.
      const bindingErrors = program.getSemanticDiagnostics(source);
      assert.deepEqual(
        bindingErrors.map((diagnostic) =>
          ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
        ),
        [],
        `invalid SDK declaration in ${file}`,
      );
    }
    for (const entry of entries) {
      visit(entry);
      const source = program.getSourceFile(entry)!;
      const symbol = checker.getSymbolAtLocation(source);
      assert.ok(symbol, `SDK entry is not a module: ${entry}`);
      const exports = new Set(checker.getExportsOfModule(symbol).map((symbol) => symbol.getName()));
      if (subpath === "./virtual-ts") {
        for (const name of consumers) assert.ok(exports.has(name), `${entry} must export ${name}`);
      } else if (subpath === "./content-mapper-helpers") {
        for (const name of coreConsumers)
          assert.ok(exports.has(name), `${entry} must export ${name}`);
      }
    }
    return {
      subpath,
      exports,
      files: [...closure].sort().map((file) => ({
        path: path.relative(packageRoot, file).split(path.sep).join("/"),
        sha256: createHash("sha256").update(read(file)).digest("hex"),
      })),
    };
  });
  return [output, `${JSON.stringify({ schema: 1, consumers, modules }, null, 2)}\n`] as const;
}

// Keep SDK source generation and this native snapshot independently checkable.
if (path.basename(process.argv[1] ?? "") === "content-mapper-helper-abi.ts") {
  const root = path.resolve(path.dirname(process.argv[1]!), "../../..");
  const [file, contents] = contentMapperHelperAbi(root, new Map());
  if (process.argv.includes("--check"))
    assert.equal(readFileSync(path.join(root, file), "utf8"), contents, `${file} is stale`);
  else writeFileSync(path.join(root, file), contents);
}
