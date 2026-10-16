import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/legacy-instance-apis-9217");
const json = (file) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
const compressed = (file) => JSON.parse(gunzipSync(fs.readFileSync(path.join(fixture, file))));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("legacy instance API packets preserve every pinned original source and complete reference finding", () => {
  const authored = json("independent.json");
  const original = compressed("originals/independent.json.gz");
  const sources = compressed("originals/sources.json.gz");
  const manifest = json("originals/manifest.json");
  const requireProvider = createRequire(
    process.env.VIZE_LEGACY_API_PROVIDER_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const requireScriptParser = createRequire(
    process.env.VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  for (const [name, key] of [
    ["eslint", "eslint"],
    ["eslint-plugin-vue", "plugin"],
    ["vue-eslint-parser", "parser"],
  ]) {
    assert.equal(requireProvider(`${name}/package.json`).version, authored.provider[key]);
  }
  const scriptParser = requireScriptParser("@typescript-eslint/parser");
  assert.equal(scriptParser.version, authored.provider.scriptParser);
  for (const rule of [
    ...Object.values(authored.provider.ruleSources),
    authored.provider.utilitySource,
  ]) {
    const directory = path.dirname(requireProvider.resolve("eslint-plugin-vue/package.json"));
    assert.equal(hash(fs.readFileSync(path.join(directory, rule.path))), rule.sha256);
  }
  const typescript = {
    ...scriptParser,
    parseForESLint(code, options) {
      return scriptParser.parseForESLint(code, {
        ...options,
        ecmaFeatures: { ...options.ecmaFeatures, jsx: false },
      });
    },
  };
  const { Linter } = requireProvider("eslint");
  const linter = new Linter();
  const config = {
    files: ["**/*.vue"],
    languageOptions: {
      parser: requireProvider("vue-eslint-parser"),
      parserOptions: {
        parser: { js: scriptParser, ts: typescript, tsx: scriptParser },
        ecmaVersion: "latest",
        sourceType: "module",
        ecmaFeatures: { jsx: true },
        extraFileExtensions: [".vue"],
      },
    },
    linterOptions: { reportUnusedDisableDirectives: "off" },
    plugins: { vue: requireProvider("eslint-plugin-vue") },
    rules: {
      "vue/no-deprecated-dollar-listeners-api": "error",
      "vue/no-deprecated-events-api": "error",
    },
  };
  assert.equal(authored.records.length, 55);
  assert.equal(sources.length, 147);
  assert.equal(original.records.length, sources.length);
  assert.equal(manifest.sourceCount, sources.length);
  assert.equal(manifest.projects.length, 18);
  const audit = json("expanded-audit.json");
  const counts = Object.fromEntries(
    Object.keys(audit.observed).map((rule) => [rule, { falsePositives: 0, falseNegatives: 0 }]),
  );
  for (const project of manifest.projects) {
    const bytes = gunzipSync(
      fs.readFileSync(
        path.join(fixture, "originals", `${project.project}-lint-divergence.json.gz`),
      ),
    );
    assert.equal(hash(bytes), project.reportSha256);
    const report = JSON.parse(bytes);
    assert.equal(report.revision, project.revision);
    assert.equal(report.project, project.project);
    for (const side of ["falsePositives", "falseNegatives"]) {
      for (const row of report.divergence[side]) {
        if (counts[row.ruleId]) counts[row.ruleId][side]++;
      }
    }
    assert.deepEqual(
      sources.filter((row) => row.project === project.project).map((row) => row.path),
      project.paths,
    );
  }
  assert.deepEqual(counts, audit.observed);
  for (const id of [...audit.ownedCoverage.expandedControls, ...audit.factoryContractControls]) {
    assert.ok(
      authored.records.some((row) => row.id === id),
      id,
    );
  }
  const gaps = json("provider-gaps.json");
  assert.equal(gaps.gaps.length, 5);
  for (const gap of gaps.gaps) {
    const packet = authored.records.find((row) => row.id === gap.caseId);
    assert.ok(packet, gap.caseId);
    assert.ok(gap.reason.length > 40);
    assert.ok(gap.nativeMessages.length > packet.messages.length);
    for (const finding of gap.nativeMessages) {
      const property = packet.source.slice(finding.start, finding.end);
      assert.ok(["$on", "$off", "$once", "$listeners"].includes(property), gap.caseId);
    }
  }
  for (const row of [...authored.records, ...original.records]) {
    const source =
      row.source ??
      sources.find((item) => item.project === row.project && item.path === row.path)?.source;
    assert.equal(typeof source, "string", row.id ?? row.path);
    assert.equal(hash(source), row.sourceSha256);
    assert.deepEqual(
      linter.verify(source, config, { filename: "Component.vue" }),
      row.messages,
      row.id ?? `${row.project}/${row.path}`,
    );
    assert.ok(row.messages.every((message) => message.fatal !== true));
  }
});

await test("Vue 3 keeps removed instance events unavailable after TypeScript erasure and optional chaining", () => {
  const requireVue = createRequire(
    process.env.VIZE_LEGACY_API_VUE_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const requireTypeScript = createRequire(
    process.env.VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const { createRenderer, version } = requireVue("vue");
  const { transpileModule, ModuleKind, ScriptTarget } = requireTypeScript("typescript");
  assert.match(version, /^3\./);
  const renderer = createRenderer({
    createComment: (text) => ({ text }),
    insert: (node, parent) => {
      parent.child = node;
    },
    remove: () => {},
    parentNode: () => null,
    nextSibling: () => null,
  });
  const failures = [];
  let handled = 0;
  let emitted = 0;
  let inspected = false;
  const app = renderer.createApp(
    {
      emits: ["probe"],
      created() {
        inspected = true;
        for (const method of ["$on", "$off", "$once"]) assert.equal(this[method], undefined);
        this.$emit("probe");
        for (const source of [
          '(this.$on as any)("probe", h)',
          '(this.$off satisfies Function)("probe", h)',
          'this.$once!("probe", h)',
          'const vm = this; ((vm.$off as any)!)("probe", h)',
        ]) {
          const { outputText } = transpileModule(source, {
            compilerOptions: { module: ModuleKind.None, target: ScriptTarget.ES2022 },
          });
          assert.throws(() => Function("h", outputText).call(this, () => handled++), TypeError);
        }
        this.$on?.("probe", () => handled++);
        this?.$once?.("probe", () => handled++);
        assert.equal(handled, 0, "optional removed calls cannot install event handlers");
      },
      render: () => null,
    },
    { onProbe: () => emitted++ },
  );
  app.config.warnHandler = (message) => failures.push(message);
  app.mount({});
  app.unmount();
  assert.equal(inspected, true);
  assert.equal(emitted, 1, "$emit remains available on the same Vue instance");
  assert.deepEqual(failures, []);
});
