// Independent input metadata from the actual installed TypeScript parser and Nuxt-generated files.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import {
  asArray,
  asRecord,
  asString,
  asStringArray,
  at,
  parseJson,
  readJson,
} from "../../json-value.ts";
import type { JsonRecord } from "../../json-value.ts";
import { lexical, save, sha } from "./javascript-workspace-project.ts";
import type { FilePin, WorkspaceContext } from "./javascript-workspace-project.ts";

/** The subset of the installed `typescript` package this evidence reads. */
interface ParseHost {
  readDirectory(
    directory: string,
    extensions: readonly string[],
    excludes?: readonly string[],
    includes?: readonly string[],
    depth?: number,
  ): string[];
}
interface TypeScriptApi {
  version: string;
  sys: ParseHost & Record<string, unknown>;
  ScriptKind: { Deferred: number };
  parseConfigFileTextToJson(filename: string, text: string): { config?: unknown; error?: unknown };
  parseJsonConfigFileContent(
    json: unknown,
    host: ParseHost,
    basePath: string,
    existingOptions: undefined,
    configFileName: string,
    resolutionStack: undefined,
    extraFileExtensions: readonly unknown[],
  ): { errors: { messageText: unknown }[]; fileNames: string[]; options: unknown };
  flattenDiagnosticMessageText(message: unknown, newLine: string): string;
  resolveModuleName(name: string, containingFile: string, options: unknown, host: unknown): unknown;
  preProcessFile(
    source: string,
    readImports: boolean,
    detectJavaScriptImports: boolean,
  ): { referencedFiles: { fileName: string }[] };
}

function loadTypeScript(root: string): TypeScriptApi {
  const module: unknown = createRequire(path.join(root, "tests/package.json"))("typescript");
  for (const name of [
    "parseConfigFileTextToJson",
    "parseJsonConfigFileContent",
    "flattenDiagnosticMessageText",
    "resolveModuleName",
    "preProcessFile",
  ])
    assert.equal(typeof at(module, name), "function", `typescript.${name}`);
  assert.equal(typeof at(module, "version"), "string");
  return module as TypeScriptApi;
}

interface LoadedConfig {
  options: JsonRecord;
  anchor: string;
  pathsOwner: string | undefined;
  config: JsonRecord;
  filename: string;
}
interface StockProgram {
  root: string;
  tsconfig: string;
  compilerOptions: JsonRecord;
  files: string[];
}
interface ExpectedFile {
  file: string;
  diagnostics: string[];
}
export interface CheckReport {
  files: ExpectedFile[];
  programs: unknown[];
  errorCount: number;
  warningCount: number;
  fileCount: number;
}
interface Declaration {
  file: string;
  source: string;
  references: { fileName: string }[];
}

const pathKeys = [
  "rootDir",
  "outDir",
  "declarationDir",
  "tsBuildInfoFile",
  "mapRoot",
  "sourceRoot",
  "outFile",
];

export function projectEvidence(root: string, context: WorkspaceContext, application: string) {
  const { artifacts, cohort } = context;
  const ts = loadTypeScript(root);
  const seen = new Map<string, LoadedConfig | "active">();
  function load(requested: string): LoadedConfig {
    const filename = fs.realpathSync(requested);
    assert.notEqual(seen.get(filename), "active", "extends cycle is outside this authored fixture");
    const cached = seen.get(filename);
    if (cached && cached !== "active") return cached;
    seen.set(filename, "active");
    const bytes = fs.readFileSync(filename, "utf8");
    const result = ts.parseConfigFileTextToJson(filename, bytes);
    assert.equal(result.error, undefined);
    const config = asRecord(result.config, filename);
    let options: JsonRecord = {},
      anchor = path.dirname(filename),
      pathsOwner: string | undefined;
    const extendsList =
      typeof config.extends === "string"
        ? [config.extends]
        : asStringArray(config.extends ?? [], "extends");
    for (const item of extendsList) {
      const extension = path.extname(item) ? item : item + ".json";
      const extended =
        item.startsWith(".") || path.isAbsolute(item)
          ? path.resolve(path.dirname(filename), extension)
          : createRequire(filename).resolve(extension);
      const parent = load(extended);
      options = { ...options, ...parent.options };
      anchor = parent.anchor;
      if (parent.pathsOwner) pathsOwner = parent.pathsOwner;
    }
    const own = asRecord(structuredClone(config.compilerOptions ?? {}), "compilerOptions");
    for (const name of pathKeys) {
      const value = own[name];
      if (typeof value === "string" && !/^[a-z][a-z0-9+.-]*:\/\//i.test(value))
        own[name] = path.resolve(path.dirname(filename), value);
    }
    for (const name of ["rootDirs", "typeRoots"])
      if (own[name])
        own[name] = asStringArray(own[name], name).map((value) =>
          path.resolve(path.dirname(filename), value),
        );
    if (own.baseUrl !== undefined) {
      const baseUrl = path.resolve(path.dirname(filename), asString(own.baseUrl, "baseUrl"));
      own.baseUrl = baseUrl;
      anchor = baseUrl;
    }
    if (own.paths !== undefined) pathsOwner = path.dirname(filename);
    options = { ...options, ...own };
    const loaded = { options, anchor, pathsOwner, config, filename };
    seen.set(filename, loaded);
    return loaded;
  }
  const shell = path.join(application, "tsconfig.json");
  const shellRead = load(shell);
  const references = shellRead.config.references;
  const configPaths =
    Array.isArray(references) && references.length
      ? references.map((item: unknown) =>
          path.resolve(application, asString(at(item, "path"), "reference.path")),
        )
      : [shell];
  const stockPrograms: StockProgram[] = [];
  const loadedPrograms = new Map<string, JsonRecord>();
  const stockOptions = new Map<string, unknown>();
  for (const filename of configPaths) {
    const loaded = load(filename);
    const effective = asRecord(structuredClone(loaded.options));
    if (effective.paths) {
      const paths = asRecord(effective.paths, "paths");
      for (const key of Object.keys(paths))
        paths[key] = asStringArray(paths[key], key).map((value) => {
          const base =
            effective.baseUrl === undefined ? loaded.pathsOwner : asString(effective.baseUrl);
          assert.ok(base !== undefined, "paths need an owning configuration");
          const selected = path.resolve(base, value);
          assert.ok(
            selected.startsWith(context.project + path.sep),
            "generated alias stays in the actual owned workspace dependency graph",
          );
          return selected;
        });
    }
    const parseHost: ParseHost = {
      ...ts.sys,
      readDirectory: (directory, extensions, excludes, includes, depth) =>
        ts.sys.readDirectory(
          directory,
          [...new Set([...extensions, ".vue"])],
          excludes,
          includes,
          depth,
        ),
    };
    const parsed = ts.parseJsonConfigFileContent(
      loaded.config,
      parseHost,
      path.dirname(loaded.filename),
      undefined,
      loaded.filename,
      undefined,
      [{ extension: ".vue", isMixedContent: true, scriptKind: ts.ScriptKind.Deferred }],
    );
    assert.deepEqual(
      parsed.errors.map((error) => ts.flattenDiagnosticMessageText(error.messageText, "\n")),
      [],
    );
    const input = [...new Set(parsed.fileNames.map((file) => fs.realpathSync(file)))].sort(lexical);
    if (input.length === 0) continue;
    const relative = (file: string): string =>
      file.startsWith(application + path.sep)
        ? path.relative(application, file).split(path.sep).join("/")
        : file;
    let common = application;
    for (const file of input)
      while (!file.startsWith(common + path.sep)) common = path.dirname(common);
    loadedPrograms.set(relative(loaded.filename), effective);
    stockOptions.set(relative(loaded.filename), parsed.options);
    stockPrograms.push({
      root: relative(common) || ".",
      tsconfig: relative(loaded.filename),
      compilerOptions: effective,
      files: input.map(relative).sort(lexical),
    });
  }
  stockPrograms.sort((a, b) => lexical(a.tsconfig, b.tsconfig));
  const schemaAlias = cohort.id === "nuxt3" ? nuxtSchemaAlias() : undefined;
  function nuxtSchemaAlias() {
    const nuxtManifest = fs.realpathSync(
      createRequire(path.join(application, "package.json")).resolve("nuxt/package.json"),
    );
    const schemaManifest = fs.realpathSync(
      createRequire(nuxtManifest).resolve("@nuxt/schema/package.json"),
    );
    const nuxt = readJson(nuxtManifest);
    const schemaTypes = path.join(
      path.dirname(nuxtManifest),
      asString(at(nuxt, "exports", "./schema", "types"), "nuxt ./schema types"),
    );
    const fallbackManifest = fs.realpathSync(
      createRequire(path.join(application, "package.json")).resolve("@nuxt/schema/package.json"),
    );
    const cohortLock = readJson(path.join(root, cohort.path, "package-lock.json"));
    return {
      nuxtManifest,
      schemaTypes,
      schemaManifest,
      package: asRecord(readJson(schemaManifest), "@nuxt/schema manifest"),
      fallback: {
        manifest: fallbackManifest,
        package: asRecord(readJson(fallbackManifest), "fallback @nuxt/schema manifest"),
        locked: at(cohortLock, "packages", "node_modules/@nuxt/schema"),
      },
      generated: stockPrograms.map((program) => ({
        tsconfig: program.tsconfig,
        paths: at(program.compilerOptions, "paths", "@nuxt/schema"),
        resolution: ts.resolveModuleName(
          "@nuxt/schema",
          schemaTypes,
          stockOptions.get(program.tsconfig),
          ts.sys,
        ),
      })),
    };
  }
  const contract = asRecord(
    readJson(
      path.join(
        root,
        "tests/_fixtures/differential/compat/javascript-workspace-products/public-input-contract.json",
      ),
    ),
    "public input contract",
  );
  assert.equal(contract.schema, "vize.compat.javascript-workspace-public-inputs");
  assert.equal(contract.version, 1);
  const membership = asRecord(at(contract, "cohorts", cohort.id), `contract.cohorts.${cohort.id}`);
  const display = (name: string): string => {
    const physical = fs.realpathSync(path.resolve(application, name));
    assert.ok(
      fs.statSync(physical).isFile(),
      "every expected public input has authored or generated bytes",
    );
    return physical.startsWith(application + path.sep)
      ? path.relative(application, physical).split(path.sep).join("/")
      : physical;
  };
  const explicitNodeDeclarations =
    cohort.id === "nuxt3" ? nodeDeclarations(ts, context, artifacts) : undefined;
  const programs = asArray(membership.programs, "membership.programs").map((value) => {
    const program = asRecord(value, "program");
    const tsconfig = asString(program.tsconfig, "program.tsconfig");
    const compilerOptions = loadedPrograms.get(tsconfig);
    assert.ok(compilerOptions, "the public program uses an actual generated or authored config");
    const files = asStringArray(program.files, "program.files").map(display);
    if (explicitNodeDeclarations) {
      assert.deepEqual(compilerOptions.types, ["node"]);
      files.push(...explicitNodeDeclarations.declarations.map((item) => display(item.file)));
    }
    return {
      root: program.root,
      tsconfig,
      compilerOptions,
      files: [...new Set(files)].sort(lexical),
    };
  });
  const names = asStringArray(membership.files, "membership.files").map(display).sort(lexical);
  assert.equal(new Set(names).size, names.length);
  const original: CheckReport = {
    files: names.map((file) => ({ file, diagnostics: [] })),
    programs,
    errorCount: 0,
    warningCount: 0,
    fileCount: names.length,
  };
  save(artifacts, "independent-project-inputs.json", {
    typescript: ts.version,
    original,
    publicContract: contract,
    stockPrograms,
    schemaAlias,
    explicitNodeDeclarations,
    configurations: [...seen].flatMap(([filename, value]) =>
      value === "active"
        ? []
        : [{ filename, bytes: fs.readFileSync(filename, "utf8"), parsed: value.config }],
    ),
  });
  if (schemaAlias) {
    assert.equal(schemaAlias.package.name, "@nuxt/schema");
    assert.equal(schemaAlias.package.version, cohort.nuxt);
    assert.equal(schemaAlias.fallback.package.name, "@nuxt/schema");
    assert.equal(schemaAlias.fallback.package.version, at(schemaAlias.fallback.locked, "version"));
    assert.equal(schemaAlias.fallback.package.version, "3.11.2");
    assert.ok(schemaAlias.generated.length > 0);
    for (const generated of schemaAlias.generated) {
      assert.deepEqual(generated.paths, [
        path.dirname(schemaAlias.schemaManifest),
        path.dirname(schemaAlias.fallback.manifest),
      ]);
      assert.equal(
        fs.realpathSync(
          asString(at(generated.resolution, "resolvedModule", "resolvedFileName"), "resolution"),
        ),
        fs.realpathSync(
          path.resolve(
            path.dirname(schemaAlias.schemaManifest),
            asString(schemaAlias.package.types, "@nuxt/schema types"),
          ),
        ),
      );
    }
  }
  return {
    original,
    stockPrograms,
    missingModuleFiles: asStringArray(membership.missingModuleFiles, "missingModuleFiles")
      .map(display)
      .sort(lexical),
  };
}

function nodeDeclarations(ts: TypeScriptApi, context: WorkspaceContext, artifacts: string) {
  const providers = readJson(path.join(artifacts, "nuxt3-type-providers.json"));
  const provider = asArray(at(providers, "graph"), "provider graph")
    .map((item) => asRecord(item, "provider"))
    .find((item) => item.name === "@types/node");
  assert.ok(provider, "the staged @types/node provider is recorded");
  const manifest = asRecord(parseJson(asString(provider.manifestBytes)), "@types/node manifest");
  assert.equal(manifest.name, "@types/node");
  assert.equal(manifest.version, context.corpus.nodeTypes.node);
  const members = asRecord(provider.members, "provider.members");
  const packageRoot = fs.realpathSync(asString(provider.target, "provider.target"));
  const entry = fs.realpathSync(path.join(packageRoot, asString(manifest.types, "types")));
  const seenDeclarations = new Map<string, Declaration>();
  const pending = [entry];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    const file = fs.realpathSync(next);
    if (seenDeclarations.has(file)) continue;
    assert.ok(file.startsWith(packageRoot + path.sep));
    assert.match(file, /\.d\.(?:ts|mts|cts)$/);
    const source = fs.readFileSync(file, "utf8");
    const relative = path.relative(packageRoot, file).split(path.sep).join("/");
    const pin: FilePin = { bytes: Buffer.byteLength(source), sha256: sha(source) };
    assert.deepEqual(members[relative], pin);
    const references = ts.preProcessFile(source, false, false).referencedFiles;
    seenDeclarations.set(file, { file, source, references });
    for (const reference of references) {
      const target = fs.realpathSync(path.resolve(path.dirname(file), reference.fileName));
      assert.ok(target.startsWith(packageRoot + path.sep));
      assert.ok(fs.statSync(target).isFile());
      pending.push(target);
    }
  }
  return {
    name: manifest.name,
    version: manifest.version,
    packageRoot,
    entry,
    declarations: [...seenDeclarations.values()].sort((a, b) => lexical(a.file, b.file)),
  };
}

export function expectedCheckMembership(
  original: CheckReport,
  missingModuleFiles: readonly string[],
  caseId: string,
): CheckReport {
  const expected = structuredClone(original);
  if (caseId === "missing-module") {
    expected.files = missingModuleFiles.map((file) => ({ file, diagnostics: [] }));
    expected.fileCount = missingModuleFiles.length;
  }
  return expected;
}
