import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";
import { compileFunction } from "node:vm";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/legacy-instance-apis-9217");
const json = (file) => JSON.parse(fs.readFileSync(path.join(fixture, file), "utf8"));
const compressed = (file) => JSON.parse(gunzipSync(fs.readFileSync(path.join(fixture, file))));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

await test("legacy instance API packets preserve every pinned original source and complete reference finding", () => {
  const authored = json("independent.json");
  const original = compressed("originals/independent.json.gz");
  const boundaries = json("template-boundaries.json");
  assert.equal(boundaries.records.length, 42);
  assert.deepEqual(boundaries.provider, authored.provider);
  assert.equal(
    boundaries.original70Sha256,
    hash(fs.readFileSync(path.join(fixture, "independent.json"))),
  );
  assert.deepEqual(
    ["qualified", "product-accuracy", "refusal"].map(
      (kind) => boundaries.records.filter((row) => row.qualification === kind).length,
    ),
    [27, 5, 10],
  );
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
  assert.equal(authored.records.length, 70);
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
  assert.equal(gaps.gaps.length, 35);
  for (const gap of gaps.gaps) {
    const packet = authored.records.find((row) => row.id === gap.caseId);
    assert.ok(packet, gap.caseId);
    assert.ok(gap.reason.length > 40);
    assert.ok(gap.nativeMessages.length > packet.messages.length);
    for (const finding of gap.nativeMessages) {
      const property = Buffer.from(packet.source).subarray(finding.start, finding.end).toString();
      assert.ok(
        ["$on", "$off", "$once", "$listeners"].includes(property.replace(/^["'`]|["'`]$/g, "")),
        gap.caseId,
      );
    }
  }
  const native = json("native.json");
  assert.equal(native.records.length, authored.records.length);
  for (const packet of native.records) {
    const source = authored.records.find((row) => row.id === packet.id);
    assert.ok(source, packet.id);
    assert.equal(source.sourceSha256, packet.sourceSha256);
    const gap = gaps.gaps.find((row) => row.caseId === packet.id);
    if (gap) assert.deepEqual(packet.messages, gap.nativeMessages, packet.id);
  }
  const ownership = json("template-ownership.json");
  assert.equal(ownership.childPr, 9251);
  assert.equal(ownership.cases.length, 70);
  assert.equal(ownership.cases.filter((row) => row.owner === "script-parent").length, 53);
  assert.equal(ownership.cases.filter((row) => row.owner === "template-child").length, 17);
  for (const [file, sha256] of Object.entries(ownership.packetSha256)) {
    assert.equal(hash(fs.readFileSync(path.join(fixture, file))), sha256);
  }
  const script = json("script-native.json");
  assert.equal(script.records.length, native.records.length);
  for (const packet of native.records) {
    const owner = ownership.cases.find((row) => row.id === packet.id);
    const projection = script.records.find((row) => row.id === packet.id);
    assert.ok(owner, packet.id);
    assert.ok(projection, packet.id);
    assert.equal(owner.sourceSha256, packet.sourceSha256);
    assert.equal(projection.sourceSha256, packet.sourceSha256);
    assert.deepEqual(
      projection.messages,
      owner.owner === "script-parent" ? packet.messages : [],
      packet.id,
    );
  }
  const provenance = compressed("provenance.json.gz");
  assert.equal(provenance.audit.rows.length, 239);
  assert.equal(provenance.packets.length, 24);
  assert.equal(
    hash(fs.readFileSync(path.join(fixture, "originals/sources.json.gz"))),
    provenance.audit.originalArchiveSha256,
  );
  assert.equal(
    hash(fs.readFileSync(path.join(fixture, "originals/manifest.json"))),
    provenance.audit.originalManifestSha256,
  );
  for (const packet of provenance.packets) {
    assert.equal(hash(Buffer.from(packet.base64, "base64")), packet.sha256);
  }
  for (const provider of Object.values(provenance.audit.providers)) {
    const packet = provenance.packets.find((row) => row.path === provider.savedPath);
    assert.ok(packet, provider.savedPath);
    assert.equal(packet.sha256, provider.sha256);
  }
  for (const packet of provenance.audit.matrixPackets) {
    assert.equal(
      hash(gunzipSync(fs.readFileSync(path.join(fixture, packet.retainedFixture)))),
      packet.sha256,
    );
  }
  const classified = {};
  for (const row of provenance.audit.rows) {
    const source = sources.find(
      (source) => source.project === row.project && source.path === row.file,
    );
    assert.ok(source, `${row.project}/${row.file}`);
    assert.equal(source.sha256, row.sourceSha256);
    for (const id of row.providerEvidenceIds) assert.ok(provenance.audit.providers[id], id);
    classified[row.classification] = (classified[row.classification] ?? 0) + 1;
  }
  assert.deepEqual(classified, provenance.audit.counts);
  for (const row of [...authored.records, ...original.records, ...boundaries.records]) {
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

await test("original Vue compiler scope and TS erasure preserve explicit reference blind spots", () => {
  const boundaries = json("template-boundaries.json");
  const requireVue = createRequire(
    process.env.VIZE_LEGACY_API_VUE_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const compiler = requireVue("@vue/compiler-dom");
  const typescript = requireVue("typescript");
  assert.equal(requireVue("vue/package.json").version, boundaries.compilerProvider.vue);
  assert.equal(
    requireVue("@vue/compiler-dom/package.json").version,
    boundaries.compilerProvider.compiler,
  );
  assert.equal(typescript.version, boundaries.compilerProvider.typescript);
  for (const source of boundaries.compilerProvider.sources) {
    const directory = path.dirname(requireVue.resolve(`${source.packageName}/package.json`));
    assert.equal(hash(fs.readFileSync(path.join(directory, source.path))), source.sha256);
  }
  assert.equal(boundaries.syntaxProof.length, 5);
  for (const expected of boundaries.syntaxProof) {
    const original = boundaries.records.find((record) => record.id === expected.id);
    assert.equal(original.qualification, "product-accuracy");
    assert.ok(original.nativePolicyReason.length > 80);
    assert.equal(expected.sourceSha256, hash(original.source));
    const diagnostics = [];
    const compiled = compiler.compile(original.source.slice(10, -11), {
      mode: "module",
      prefixIdentifiers: true,
      expressionPlugins: ["typescript"],
      onError(error) {
        diagnostics.push({
          name: error.name,
          message: error.message,
          code: error.code,
          loc: error.loc,
        });
      },
    });
    const erased = typescript.transpileModule(compiled.code, {
      compilerOptions: {
        target: typescript.ScriptTarget.ES2022,
        module: typescript.ModuleKind.ESNext,
      },
      reportDiagnostics: true,
    });
    assert.deepEqual(
      {
        id: original.id,
        sourceSha256: hash(original.source),
        code: compiled.code,
        preamble: compiled.preamble ?? "",
        map: compiled.map ?? null,
        diagnostics,
        typescriptOutput: erased.outputText,
        typescriptDiagnostics: (erased.diagnostics ?? []).map((diagnostic) => ({
          code: diagnostic.code,
          category: diagnostic.category,
          start: diagnostic.start ?? null,
          length: diagnostic.length ?? null,
          messageText: diagnostic.messageText,
        })),
      },
      expected,
      original.id,
    );
    assert.match(erased.outputText, /_ctx\.\$listeners/);
    assert.deepEqual(expected.diagnostics, []);
    assert.deepEqual(expected.typescriptDiagnostics, []);
  }
});

await test("Vue 3 keeps removed APIs unavailable on public instances and Options-returning factories", () => {
  const requireVue = createRequire(
    process.env.VIZE_LEGACY_API_VUE_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const requireTypeScript = createRequire(
    process.env.VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST ??
      path.join(root, "tools/benchmarks/scripts/package.json"),
  );
  const runtime = json("runtime.json");
  const { createRenderer, h, version } = requireVue("vue");
  const requireDom = createRequire(requireVue.resolve("@vue/runtime-dom/package.json"));
  const runtimePath = path.join(
    path.dirname(requireDom.resolve("@vue/runtime-core/package.json")),
    runtime.provider.source.path,
  );
  assert.equal(version, runtime.provider.vue);
  assert.equal(hash(fs.readFileSync(runtimePath)), runtime.provider.source.sha256);
  const typescript = requireTypeScript("typescript");
  assert.equal(typescript.version, runtime.provider.typescript);
  const { transpileModule, ModuleKind, ScriptTarget } = typescript;
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
  const receipt = {
    mountedOptionsFactory: false,
    relations: [],
    wrappedFailures: 0,
    optionalHandlers: 0,
    emitEvents: 0,
    foreignHandlers: 0,
  };
  function erasedCall(instance, source) {
    const { outputText } = transpileModule(source, {
      compilerOptions: { module: ModuleKind.None, target: ScriptTarget.ES2022 },
    });
    assert.throws(
      () => compileFunction(outputText, ["h"]).call(instance, () => receipt.optionalHandlers++),
      TypeError,
    );
    receipt.wrappedFailures++;
  }
  const Child = {
    created() {
      receipt.relations.push({
        kind: "child",
        rootIsSelf: this.$root === this,
        parentIsRoot: this.$parent === this.$root,
        rootOn: typeof this.$root.$on,
        parentOn: typeof this.$parent.$on,
        rootEmit: typeof this.$root.$emit,
      });
      erasedCall(this, '(this.$root.$on as any)("probe", h)');
      erasedCall(this, '(this.$parent!.$off satisfies Function)("probe", h)');
      this?.$parent?.$root?.$on?.("probe", () => receipt.optionalHandlers++);
    },
    render: () => null,
  };
  function MdComponent(options) {
    return { ...options };
  }
  const app = renderer.createApp(
    new MdComponent({
      emits: ["probe"],
      created() {
        receipt.mountedOptionsFactory = true;
        receipt.relations.push({
          kind: "root",
          rootIsSelf: this.$root === this,
          parent: this.$parent,
          rootOn: typeof this.$root.$on,
        });
        for (const method of ["$on", "$off", "$once"]) assert.equal(this[method], undefined);
        this.$emit("probe");
        for (const source of [
          '(this.$on as any)("probe", h)',
          '(this.$off satisfies Function)("probe", h)',
          'this.$once!("probe", h)',
          'const vm = this; ((vm.$off as any)!)("probe", h)',
        ])
          erasedCall(this, source);
        this.$on?.("probe", () => receipt.optionalHandlers++);
        this?.$once?.("probe", () => receipt.optionalHandlers++);
      },
      render: () => h(Child),
    }),
    { onProbe: () => receipt.emitEvents++ },
  );
  app.config.warnHandler = (message) => failures.push(message);
  app.mount({});
  app.unmount();
  const external = {
    $on(_event, handler) {
      handler();
    },
  };
  external.$on("probe", () => receipt.foreignHandlers++);
  assert.deepEqual(receipt, runtime.expected);
  assert.deepEqual(failures, []);
});
