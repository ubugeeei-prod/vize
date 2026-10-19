// Independent input metadata from the actual installed TypeScript parser and Nuxt-generated files.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { lexical, save, sha } from "./javascript-workspace-project.mjs";

export function projectEvidence(root, context, application) {
  const { artifacts, cohort } = context;
  const ts = createRequire(path.join(root, "tests/package.json"))("typescript");
  const seen = new Map();
  const pathKeys = [
    "rootDir",
    "outDir",
    "declarationDir",
    "tsBuildInfoFile",
    "mapRoot",
    "sourceRoot",
    "outFile",
  ];
  function load(filename) {
    filename = fs.realpathSync(filename);
    assert.notEqual(seen.get(filename), "active", "extends cycle is outside this authored fixture");
    const cached = seen.get(filename);
    if (cached) return cached;
    seen.set(filename, "active");
    const bytes = fs.readFileSync(filename, "utf8");
    const result = ts.parseConfigFileTextToJson(filename, bytes);
    assert.equal(result.error, undefined);
    const config = result.config;
    let options = {},
      anchor = path.dirname(filename),
      pathsOwner;
    for (const item of typeof config.extends === "string"
      ? [config.extends]
      : (config.extends ?? [])) {
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
    const own = structuredClone(config.compilerOptions ?? {});
    for (const name of pathKeys)
      if (typeof own[name] === "string" && !/^[a-z][a-z0-9+.-]*:\/\//i.test(own[name]))
        own[name] = path.resolve(path.dirname(filename), own[name]);
    for (const name of ["rootDirs", "typeRoots"])
      if (own[name])
        own[name] = own[name].map((value) => path.resolve(path.dirname(filename), value));
    if (own.baseUrl !== undefined) {
      own.baseUrl = path.resolve(path.dirname(filename), own.baseUrl);
      anchor = own.baseUrl;
    }
    if (own.paths !== undefined) pathsOwner = path.dirname(filename);
    options = { ...options, ...own };
    const loaded = { options, anchor, pathsOwner, config, filename };
    seen.set(filename, loaded);
    return loaded;
  }
  const shell = path.join(application, "tsconfig.json");
  const shellRead = load(shell);
  const configPaths = shellRead.config.references?.length
    ? shellRead.config.references.map((item) => path.resolve(application, item.path))
    : [shell];
  const stockPrograms = [];
  const loadedPrograms = new Map();
  const stockOptions = new Map();
  for (const filename of configPaths) {
    const loaded = load(filename);
    const effective = structuredClone(loaded.options);
    if (effective.paths)
      for (const key of Object.keys(effective.paths))
        effective.paths[key] = effective.paths[key].map((value) => {
          const selected = path.resolve(effective.baseUrl ?? loaded.pathsOwner, value);
          assert.ok(
            selected.startsWith(context.project + path.sep),
            "generated alias stays in the actual owned workspace dependency graph",
          );
          return selected;
        });
    const parseHost = {
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
    const relative = (file) =>
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
  let schemaAlias;
  if (cohort.id === "nuxt3") {
    const nuxtManifest = fs.realpathSync(
      createRequire(path.join(application, "package.json")).resolve("nuxt/package.json"),
    );
    const schemaManifest = fs.realpathSync(
      createRequire(nuxtManifest).resolve("@nuxt/schema/package.json"),
    );
    const nuxt = JSON.parse(fs.readFileSync(nuxtManifest, "utf8"));
    const schemaTypes = path.join(path.dirname(nuxtManifest), nuxt.exports["./schema"].types);
    const fallbackManifest = fs.realpathSync(
      createRequire(path.join(application, "package.json")).resolve("@nuxt/schema/package.json"),
    );
    const cohortLock = JSON.parse(
      fs.readFileSync(path.join(root, cohort.path, "package-lock.json"), "utf8"),
    );
    schemaAlias = {
      nuxtManifest,
      schemaTypes,
      schemaManifest,
      package: JSON.parse(fs.readFileSync(schemaManifest, "utf8")),
      fallback: {
        manifest: fallbackManifest,
        package: JSON.parse(fs.readFileSync(fallbackManifest, "utf8")),
        locked: cohortLock.packages["node_modules/@nuxt/schema"],
      },
      generated: stockPrograms.map((program) => ({
        tsconfig: program.tsconfig,
        paths: program.compilerOptions.paths["@nuxt/schema"],
        resolution: ts.resolveModuleName(
          "@nuxt/schema",
          schemaTypes,
          stockOptions.get(program.tsconfig),
          ts.sys,
        ),
      })),
    };
  }
  const contract = JSON.parse(
    fs.readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/compat/javascript-workspace-products/public-input-contract.json",
      ),
      "utf8",
    ),
  );
  assert.equal(contract.schema, "vize.compat.javascript-workspace-public-inputs");
  assert.equal(contract.version, 1);
  const membership = contract.cohorts[cohort.id];
  const display = (name) => {
    const physical = fs.realpathSync(path.resolve(application, name));
    assert.ok(
      fs.statSync(physical).isFile(),
      "every expected public input has authored or generated bytes",
    );
    return physical.startsWith(application + path.sep)
      ? path.relative(application, physical).split(path.sep).join("/")
      : physical;
  };
  let explicitNodeDeclarations;
  if (cohort.id === "nuxt3") {
    const provider = JSON.parse(
      fs.readFileSync(path.join(artifacts, "nuxt3-type-providers.json"), "utf8"),
    ).graph.find((item) => item.name === "@types/node");
    const manifest = JSON.parse(provider.manifestBytes);
    assert.equal(manifest.name, "@types/node");
    assert.equal(manifest.version, context.corpus.nodeTypes.node);
    const packageRoot = fs.realpathSync(provider.target);
    const entry = fs.realpathSync(path.join(packageRoot, manifest.types));
    const seenDeclarations = new Map();
    const pending = [entry];
    while (pending.length) {
      const file = fs.realpathSync(pending.pop());
      if (seenDeclarations.has(file)) continue;
      assert.ok(file.startsWith(packageRoot + path.sep));
      assert.match(file, /\.d\.(?:ts|mts|cts)$/);
      const source = fs.readFileSync(file, "utf8");
      const relative = path.relative(packageRoot, file).split(path.sep).join("/");
      assert.deepEqual(provider.members[relative], {
        bytes: Buffer.byteLength(source),
        sha256: sha(source),
      });
      const references = ts.preProcessFile(source, false, false).referencedFiles;
      seenDeclarations.set(file, { file, source, references });
      for (const reference of references) {
        const target = fs.realpathSync(path.resolve(path.dirname(file), reference.fileName));
        assert.ok(target.startsWith(packageRoot + path.sep));
        assert.ok(fs.statSync(target).isFile());
        pending.push(target);
      }
    }
    explicitNodeDeclarations = {
      name: manifest.name,
      version: manifest.version,
      packageRoot,
      entry,
      declarations: [...seenDeclarations.values()].sort((a, b) => lexical(a.file, b.file)),
    };
  }
  const programs = membership.programs.map((program) => {
    const compilerOptions = loadedPrograms.get(program.tsconfig);
    assert.ok(compilerOptions, "the public program uses an actual generated or authored config");
    const files = program.files.map(display);
    if (explicitNodeDeclarations) {
      assert.deepEqual(compilerOptions.types, ["node"]);
      files.push(...explicitNodeDeclarations.declarations.map((item) => display(item.file)));
    }
    return {
      root: program.root,
      tsconfig: program.tsconfig,
      compilerOptions,
      files: [...new Set(files)].sort(lexical),
    };
  });
  const names = membership.files.map(display).sort(lexical);
  assert.equal(new Set(names).size, names.length);
  const original = {
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
    configurations: [...seen]
      .filter(([, value]) => value !== "active")
      .map(([filename, value]) => ({
        filename,
        bytes: fs.readFileSync(filename, "utf8"),
        parsed: value.config,
      })),
  });
  if (schemaAlias) {
    assert.equal(schemaAlias.package.name, "@nuxt/schema");
    assert.equal(schemaAlias.package.version, cohort.nuxt);
    assert.equal(schemaAlias.fallback.package.name, "@nuxt/schema");
    assert.equal(schemaAlias.fallback.package.version, schemaAlias.fallback.locked.version);
    assert.equal(schemaAlias.fallback.package.version, "3.11.2");
    assert.ok(schemaAlias.generated.length > 0);
    for (const generated of schemaAlias.generated) {
      assert.deepEqual(generated.paths, [
        path.dirname(schemaAlias.schemaManifest),
        path.dirname(schemaAlias.fallback.manifest),
      ]);
      assert.equal(
        fs.realpathSync(generated.resolution.resolvedModule.resolvedFileName),
        fs.realpathSync(
          path.resolve(path.dirname(schemaAlias.schemaManifest), schemaAlias.package.types),
        ),
      );
    }
  }
  return {
    original,
    stockPrograms,
    missingModuleFiles: membership.missingModuleFiles.map(display).sort(lexical),
  };
}

export function expectedCheckMembership(original, missingModuleFiles, caseId) {
  const expected = structuredClone(original);
  if (caseId === "missing-module") {
    expected.files = missingModuleFiles.map((file) => ({ file, diagnostics: [] }));
    expected.fileCount = missingModuleFiles.length;
  }
  return expected;
}
