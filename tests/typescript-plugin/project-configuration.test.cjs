const assert = require("node:assert/strict");
const { test } = require("node:test");
const { createMappedLanguageService } = require("../../npm/typescript-plugin/language-service.cjs");
const ts = require("../node_modules/typescript");

void test("one project fingerprints its native closure once per epoch and disposes exact watches", (context) => {
  const fs = require("node:fs"),
    os = require("node:os"),
    path = require("node:path");
  const { installVueVirtualModules } = require("../../npm/typescript-plugin/virtual-modules.cjs");
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-project-closure-"));
  const nativeRoot = path.join(root, "native");
  fs.mkdirSync(nativeRoot);
  fs.writeFileSync(
    path.join(nativeRoot, "package.json"),
    JSON.stringify({ name: "@vizejs/native", main: "index.cjs" }),
  );
  fs.writeFileSync(path.join(nativeRoot, "index.cjs"), "module.exports = {};\n");
  fs.copyFileSync(
    require.resolve("../../npm/native/language-projection.js"),
    path.join(nativeRoot, "language-projection.js"),
  );
  const native = require(path.join(nativeRoot, "index.cjs"));
  const config = path.join(root, "tsconfig.json"),
    base = path.join(root, "inherited.json"),
    missing = path.join(root, "preferred.json");
  const files = new Map([
    [config, "root"],
    [base, "loose"],
  ]);
  const reads = new Map(),
    watches = new Map(),
    closed = [];
  const sources = new Map(
    Array.from({ length: 25 }, (_, index) => [
      path.join(root, `${index}.vue`),
      `<template>${index}</template>`,
    ]),
  );
  let version = 0,
    resolverCalls = 0,
    generationCalls = 0,
    refreshes = 0,
    malformed = false;
  native.getVueLanguageProjectMetadata = () => {
    resolverCalls++;
    if (malformed) throw new Error("Invalid inherited configuration");
    return {
      configIdentity: files.get(missing) || files.get(base),
      watchedFiles: [config, base, missing],
    };
  };
  native.generateVueLanguageCode = () => {
    generationCalls++;
    return JSON.stringify({
      text: `export default ${JSON.stringify(files.get(missing) || files.get(base))};`,
      extension: ".tsx",
      mappings: [],
      diagnostics: [],
    });
  };
  const readFile = (file) => {
    reads.set(file, (reads.get(file) || 0) + 1);
    return files.get(file);
  };
  const host = {
    readFile,
    fileExists: (file) => files.has(file) || sources.has(file),
    getScriptSnapshot: (file) =>
      sources.has(file) ? ts.ScriptSnapshot.fromString(sources.get(file)) : undefined,
    getCompilationSettings: () => ({ configFilePath: config }),
    getCurrentDirectory: () => root,
    getProjectVersion: () => String(version),
  };
  const originalVersion = host.getProjectVersion;
  const support = installVueVirtualModules(ts, {
    languageServiceHost: host,
    config: { nativePath: nativeRoot },
    project: { refreshDiagnostics: () => refreshes++ },
    serverHost: {
      watchFile(file, callback, pollingInterval) {
        assert.equal(
          pollingInterval,
          250,
          "missing-file host polling has an explicit numeric interval",
        );
        assert(!watches.has(file), "each observed input has one live watcher");
        watches.set(file, callback);
        return {
          close() {
            closed.push(file);
            watches.delete(file);
          },
        };
      },
    },
  });
  const queryAll = () => [...sources.keys()].map((file) => support.get(file));
  const crypto = require("node:crypto");
  const originalHash = crypto.createHash;
  let hashCalls = 0;
  crypto.createHash = (...args) => {
    hashCalls++;
    return originalHash(...args);
  };
  try {
    queryAll();
    queryAll();
    assert.equal(resolverCalls, 2);
    assert.equal(generationCalls, 25);
    assert.deepEqual([...watches.keys()], [config, base, missing]);
    assert.deepEqual(
      [...reads.values()],
      [1, 1, 1],
      "50 SFC queries share three known input reads",
    );
    assert.equal(
      hashCalls,
      27,
      "25 projection version digests plus two readable configuration digests",
    );
    version++;
    queryAll();
    assert.equal(resolverCalls, 2, "an unchanged project epoch does not reload the resolver");
    assert.equal(generationCalls, 25);
    assert.deepEqual([...reads.values()], [2, 2, 2]);
    assert.equal(
      hashCalls,
      29,
      "an unchanged epoch hashes each readable input once, not once per SFC",
    );
    context.diagnostic(
      JSON.stringify({
        scope: "JavaScript cache policy, controlled provider; not native or wall-time throughput",
        sfcs: 25,
        queries: 75,
        resolverCalls,
        generations: generationCalls,
        knownInputReads: [...reads.values()],
        hashCalls,
      }),
    );
    watches.get(base)(base, ts.FileWatcherEventKind.Changed);
    queryAll();
    assert.equal(
      resolverCalls,
      2,
      "unchanged watcher events retain the canonical resolver snapshot",
    );
    assert.equal(generationCalls, 25);
    const originalProject = host.getProjectVersion();
    files.set(base, "strict");
    watches.get(base)(base, ts.FileWatcherEventKind.Changed);
    assert.equal(refreshes, 2);
    assert.notEqual(
      host.getProjectVersion(),
      originalProject,
      "a config event invalidates the actual host project version",
    );
    queryAll();
    assert.equal(resolverCalls, 3);
    assert.equal(generationCalls, 50);
    assert.deepEqual([...reads.values()], [4, 4, 4]);
    files.set(missing, "preferred");
    watches.get(missing)(missing, ts.FileWatcherEventKind.Created);
    const preferred = queryAll();
    assert.equal(resolverCalls, 4);
    assert(
      preferred.every((document) => document.projection.text === 'export default "preferred";'),
    );
    files.delete(missing);
    watches.get(missing)(missing, ts.FileWatcherEventKind.Deleted);
    assert(queryAll().every((document) => document.projection.text === 'export default "strict";'));
    assert.equal(resolverCalls, 5);
    malformed = true;
    files.set(base, "invalid");
    version++;
    assert(queryAll().every((document) => document.error === "Invalid inherited configuration"));
    assert.equal(generationCalls, 100, "failed configuration never calls the projection generator");
    malformed = false;
    files.set(base, "strict");
    version++;
    assert(queryAll().every((document) => !document.error));
    assert.equal(resolverCalls, 7, "a rejected snapshot can recover on the next project epoch");
    const resolver = native.getVueLanguageProjectMetadata;
    let changeDuringResolve = true;
    native.getVueLanguageProjectMetadata = () => {
      const result = resolver();
      if (changeDuringResolve) {
        changeDuringResolve = false;
        files.set(base, "concurrent");
      }
      return result;
    };
    files.set(config, "ordinary-root-change");
    watches.get(config)(config, ts.FileWatcherEventKind.Changed);
    queryAll();
    watches.get(base)(base, ts.FileWatcherEventKind.Changed);
    assert(
      queryAll().every((document) => document.projection.text === 'export default "concurrent";'),
      "a second known input changing during metadata resolution must invalidate the prior projection",
    );
    const discovered = path.join(root, "new-preferred.json");
    let createDuringResolve = true;
    native.getVueLanguageProjectMetadata = () => {
      resolverCalls++;
      const result = {
        configIdentity: `closure:${files.get(discovered) || "fallback"}`,
        watchedFiles: [config, base, missing, discovered],
      };
      if (createDuringResolve) {
        createDuringResolve = false;
        files.set(discovered, "created");
      }
      return result;
    };
    native.generateVueLanguageCode = () => {
      generationCalls++;
      return JSON.stringify({
        text: `export default ${JSON.stringify(files.get(discovered) || files.get(base))};`,
        extension: ".tsx",
        mappings: [],
        diagnostics: [],
      });
    };
    const beforeDiscovery = resolverCalls;
    files.set(config, "discover-closure");
    watches.get(config)(config, ts.FileWatcherEventKind.Changed);
    const createdDuringCapture = queryAll();
    assert.equal(
      resolverCalls - beforeDiscovery,
      2,
      "new inputs are confirmed after their first fingerprint",
    );
    assert(
      createdDuringCapture.every(
        (document) => document.projection.text === 'export default "created";',
      ),
    );
    assert.equal(
      JSON.parse(createdDuringCapture[0].configuration)[2],
      "closure:created",
      "the admitted identity must consume the newly fingerprinted bytes",
    );
    files.delete(discovered);
    watches.get(discovered)(discovered, ts.FileWatcherEventKind.Deleted);
    assert(
      queryAll().every((document) => document.projection.text === 'export default "concurrent";'),
      "returning to the initial fallback cannot reuse a wrongly labeled projection",
    );
    const stableResolver = native.getVueLanguageProjectMetadata;
    const unstablePaths = [];
    let captures = 0;
    native.getVueLanguageProjectMetadata = () => {
      const input = path.join(root, `unstable-${++captures}.json`);
      unstablePaths.push(input);
      return { configIdentity: `unstable:${captures}`, watchedFiles: [config, input] };
    };
    files.set(config, "unstable-closure");
    watches.get(config)(config, ts.FileWatcherEventKind.Changed);
    const beforeFailureGenerations = generationCalls;
    const rejected = queryAll();
    const unstableError = "Project configuration dependencies changed during three captures";
    assert.equal(captures, 3);
    assert.equal(
      generationCalls,
      beforeFailureGenerations,
      "actively expanding captures never generate a partial projection",
    );
    assert(
      rejected.every(
        (document) => document.error === unstableError && document.projection === undefined,
      ),
    );
    assert.equal(JSON.parse(rejected[0].configuration)[2], unstableError);
    const failureService = createMappedLanguageService(
      ts,
      {
        getSemanticDiagnostics() {
          throw new Error("an unstable project must not query generated diagnostics");
        },
      },
      support,
    );
    const failure = failureService.getSemanticDiagnostics([...sources.keys()][0]);
    assert.equal(failure[0].file.text, [...sources.values()][0]);
    assert.deepEqual(
      failure.map(({ file: _file, ...packet }) => packet),
      [
        {
          start: 0,
          length: 0,
          code: 100003,
          source: "vize",
          messageText: unstableError,
          category: ts.DiagnosticCategory.Error,
        },
      ],
    );
    native.getVueLanguageProjectMetadata = stableResolver;
    watches.get(config)(config, ts.FileWatcherEventKind.Changed);
    assert(queryAll().every((document) => !document.error));
    assert.deepEqual([...watches.keys()], [config, base, missing, discovered]);
    support.dispose();
    assert.equal(watches.size, 0);
    assert.deepEqual(closed, [...unstablePaths, config, base, missing, discovered]);
    assert.equal(host.getProjectVersion, originalVersion);
    assert.equal(support.snapshots.size, 0);
    const file = [...sources.keys()][0];
    const authored = "// 😀\r\nconst missing = Missing;\r\n";
    const prefix = "declare const generated: unknown;\n";
    sources.set(file, authored);
    native.generateVueLanguageCode = () =>
      JSON.stringify({
        text: prefix + authored,
        extension: ".tsx",
        mappings: [
          [
            Buffer.byteLength(prefix),
            Buffer.byteLength(authored),
            0,
            Buffer.byteLength(authored),
            0,
            0xfffff,
          ],
        ],
        diagnostics: [],
      });
    let rawDisposals = 0,
      outerCalls = 0;
    const raw = {
      getSyntacticDiagnostics: () => [],
      getSemanticDiagnostics: () => [
        {
          start: prefix.length + authored.indexOf("Missing"),
          length: 7,
          code: 2339,
          messageText: "authored missing name",
          category: ts.DiagnosticCategory.Error,
        },
      ],
      dispose: () => rawDisposals++,
    };
    const activate = (languageService) =>
      require("../../npm/typescript-plugin/index.cjs")({ typescript: ts }).create({
        languageService,
        languageServiceHost: host,
        config: { nativePath: nativeRoot },
        serverHost: {
          watchFile(file, callback) {
            watches.set(file, callback);
            return {
              close() {
                watches.delete(file);
              },
            };
          },
        },
      });
    const first = activate(raw);
    const expected = first.getSemanticDiagnostics(file);
    assert.equal(expected[0].start, authored.indexOf("Missing"));
    assert.equal(expected[0].messageText, "authored missing name");
    assert.equal(
      activate(first),
      first,
      "a real configured-project reactivation does not map the same packet twice",
    );
    const foreign = {
      ...first,
      getSemanticDiagnostics(name) {
        outerCalls++;
        return first.getSemanticDiagnostics(name);
      },
    };
    const reloaded = activate(foreign);
    assert.equal(
      reloaded,
      foreign,
      "another plugin's complete outer wrapper stays in the live chain",
    );
    assert.deepEqual(reloaded.getSemanticDiagnostics(file), expected);
    assert.equal(outerCalls, 1);
    reloaded.dispose();
    assert.equal(rawDisposals, 1);
    assert.equal(watches.size, 0);
    const reopened = activate(raw);
    assert.notEqual(reopened, raw, "a fresh project lifetime owns a fresh mapper");
    assert.deepEqual(reopened.getSemanticDiagnostics(file), expected);
    reopened.dispose();
    assert.equal(rawDisposals, 2);
    assert.equal(watches.size, 0);
  } finally {
    crypto.createHash = originalHash;
    support.dispose();
    fs.rmSync(root, { recursive: true, force: true });
  }
});
