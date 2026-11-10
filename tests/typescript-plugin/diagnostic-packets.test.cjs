const assert = require("node:assert/strict");
const { mkdtempSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { test } = require("node:test");
const { createMappedLanguageService } = require("../../npm/typescript-plugin/language-service.cjs");
const helper = require("../../npm/native/language-projection.js");
const ts = require("../node_modules/typescript");

void test("one project version shares the complete diagnostic pass and invalidates atomically", () => {
  const file = "/src/Cached.vue";
  const source = "<script>xy</script>";
  const syntax = { start: 0, length: 1, code: 1109, category: 1, messageText: "syntax" };
  const semantic = { start: 1, length: 1, code: 2322, category: 1, messageText: "semantic" };
  const suggestion = { start: 0, length: 2, code: 6133, category: 2, messageText: "unused" };
  const projection = {
    text: "xy",
    mappings: [[0, 2, 8, 2, 0, 0xfffff]],
    diagnostics: [],
  };
  let document = { source, projection };
  let version = 1;
  const calls = { syntax: 0, semantic: 0, suggestion: 0 };
  const service = createMappedLanguageService(
    ts,
    {
      getSyntacticDiagnostics() {
        calls.syntax++;
        return [syntax];
      },
      getSemanticDiagnostics() {
        calls.semantic++;
        return [semantic];
      },
      getSuggestionDiagnostics() {
        calls.suggestion++;
        return [suggestion];
      },
    },
    {
      engine: helper,
      get: () => document,
      projectVersion: () => String(version),
    },
  );

  assert.deepEqual(
    service.getSyntacticDiagnostics(file).map(({ code }) => code),
    [1109],
  );
  assert.deepEqual(
    service.getSemanticDiagnostics(file).map(({ code }) => code),
    [2322],
  );
  assert.deepEqual(calls, { syntax: 1, semantic: 1, suggestion: 0 });
  assert.deepEqual(
    service.getSuggestionDiagnostics(file).map(({ code }) => code),
    [6133],
  );
  assert.deepEqual(calls, { syntax: 1, semantic: 1, suggestion: 1 });

  version++;
  assert.deepEqual(
    service.getSemanticDiagnostics(file).map(({ code }) => code),
    [2322],
  );
  assert.deepEqual(
    service.getSyntacticDiagnostics(file).map(({ code }) => code),
    [1109],
  );
  assert.deepEqual(calls, { syntax: 2, semantic: 2, suggestion: 1 });

  // A new projection identity in the same project version is a new packet too.
  document = { source, projection: { ...projection } };
  assert.deepEqual(
    service.getSyntacticDiagnostics(file).map(({ code }) => code),
    [1109],
  );
  assert.deepEqual(calls, { syntax: 3, semantic: 3, suggestion: 1 });
});

void test("opt-in diagnostic profiling records raw phase evidence without changing packets", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-plugin-diagnostics-"));
  const output = join(directory, "diagnostics.json");
  const prior = process.env.VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE;
  process.env.VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE = output;
  try {
    const file = "/src/Profile.vue";
    const source = "<script>xy</script>";
    const syntax = { start: 0, length: 1, code: 1109, category: 1, messageText: "syntax" };
    const semantic = { start: 1, length: 1, code: 2322, category: 1, messageText: "semantic" };
    const projection = {
      text: "xy",
      mappings: [[0, 2, 8, 2, 0, 0xfffff]],
      diagnostics: [],
    };
    const document = { source, projection };
    const service = createMappedLanguageService(
      ts,
      {
        getSyntacticDiagnostics: () => [syntax],
        getSemanticDiagnostics: () => [semantic],
      },
      { engine: helper, get: () => document, projectVersion: () => "1" },
    );
    assert.deepEqual(
      service.getSyntacticDiagnostics(file).map(({ code }) => code),
      [1109],
    );
    assert.deepEqual(
      service.getSemanticDiagnostics(file).map(({ code }) => code),
      [2322],
    );
    service.dispose();

    const profile = JSON.parse(readFileSync(output, "utf8"));
    assert.equal(profile.contract, "vize-typescript-plugin-diagnostics-v1");
    assert.deepEqual(
      profile.events.map(
        ({
          method,
          projected,
          cacheHit,
          projectVersion,
          sourceLength,
          generatedLength,
          resultCount,
        }) => ({
          method,
          projected,
          cacheHit,
          projectVersion,
          sourceLength,
          generatedLength,
          resultCount,
        }),
      ),
      [
        {
          method: "getSyntacticDiagnostics",
          projected: true,
          cacheHit: false,
          projectVersion: "1",
          sourceLength: source.length,
          generatedLength: projection.text.length,
          resultCount: 1,
        },
        {
          method: "getSemanticDiagnostics",
          projected: true,
          cacheHit: true,
          projectVersion: "1",
          sourceLength: source.length,
          generatedLength: projection.text.length,
          resultCount: 1,
        },
      ],
    );
    assert.deepEqual(Object.keys(profile.events[0].phasesMs), [
      "ensure",
      "projection",
      "authoredSource",
      "version",
      "typescriptSyntactic",
      "typescriptSemantic",
      "directives",
      "mapping",
      "completedVersion",
    ]);
    assert.deepEqual(Object.keys(profile.events[1].phasesMs), [
      "ensure",
      "projection",
      "authoredSource",
      "version",
    ]);
    for (const event of profile.events) {
      assert.ok(event.totalMs >= 0);
      assert.ok(Object.values(event.phasesMs).every((value) => value >= 0));
    }
  } finally {
    if (prior === undefined) delete process.env.VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE;
    else process.env.VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE = prior;
    rmSync(directory, { recursive: true, force: true });
  }
});
