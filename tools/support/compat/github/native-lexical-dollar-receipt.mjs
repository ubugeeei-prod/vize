import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { digest, validateMetadata } from "./native-lexical-dollar-metadata.mjs";

export const nativeLaw =
  "batch::type_checker::tests::recent_issues::lexical_dollar_slots::lexical_dollar_slots_preserve_complete_native_diagnostics_and_repairs";
export const producerLaws = [
  "lexical_scope::lexical_dollar_scopes_keep_only_real_instance_reads_and_exact_authored_ranges",
  "lexical_scope::permissive_dollar_scopes_do_not_hoist_lexical_slot_loop_or_handler_names",
  "the_permissive_form_declares_the_name_once_with_an_any_fallback",
  "the_permissive_form_maps_the_single_declaration_to_the_first_occurrence",
  "the_strict_form_maps_each_access_back_to_its_authored_occurrence",
  "the_strict_form_reads_every_occurrence_off_the_template_context",
].map((name) => `virtual_ts::scope::globals::instance::tests::${name}`);
const cases = [
  ["valid-strict", "valid.vue.txt", true],
  ["wrong-strict", "wrong.vue.txt", true],
  ["wrong-permissive", "wrong.vue.txt", false],
  ["escape-strict", "escape.vue.txt", true],
  ["foreign-strict", "foreign.vue.txt", true],
  ["loop-source-strict", "loop-source.vue.txt", true],
  ["globals-strict", "globals.vue.txt", true],
  ["globals-permissive", "globals.vue.txt", false],
  ["repaired-strict", "valid.vue.txt", true],
];
const bytes = (value) => {
  assert.ok(
    Array.isArray(value) &&
      value.every((byte) => Number.isInteger(byte) && byte >= 0 && byte <= 255),
  );
  return Buffer.from(value);
};
const utf8 = (buffer) => {
  const text = buffer.toString("utf8");
  assert.deepEqual(Buffer.from(text), buffer);
  return text;
};

function completeMap(map, code, authored) {
  assert.equal(typeof map, "string");
  assert.ok(map.startsWith("CompositeSourceMap {\n") && map.endsWith("\n}"));
  for (const required of [
    /sfc_map: Some\(/u,
    /SfcSourceMap \{/u,
    /projection: ProjectionMapping \{/u,
    /spans: \[/u,
    /meta: \[/u,
    /semantic_links: \[/u,
    /prop_default_key_links: \[/u,
    /authored_base: 0,/u,
    /authored_disjoint: false,/u,
    /blocks: \[/u,
    /import_map: ImportSourceMap \{/u,
    /adjustments: \[/u,
  ])
    assert.match(map, required);
  const stack = [];
  let quoted = false;
  let escaped = false;
  for (const character of map) {
    if (quoted) {
      if (escaped) escaped = false;
      else if (character === "\\") escaped = true;
      else if (character === '"') quoted = false;
      continue;
    }
    if (character === '"') quoted = true;
    else if ("([{".includes(character)) stack.push(character);
    else if (")]}".includes(character))
      assert.equal(stack.pop(), { ")": "(", "]": "[", "}": "{" }[character]);
  }
  assert.equal(quoted, false);
  assert.equal(stack.length, 0);
  const spans = [...map.matchAll(/gen_range: (\d+)\.\.(\d+),\s+src_range: (\d+)\.\.(\d+),/gu)];
  assert.ok(spans.length > 0);
  for (const [, gs, ge, ss, se] of spans) {
    assert.ok(+gs <= +ge && +ge <= Buffer.byteLength(code));
    assert.ok(+ss <= +se && +se <= Buffer.byteLength(authored));
  }
  const blocks = [
    ...map.matchAll(/SfcBlockRange \{\s+start: (\d+),\s+end: (\d+),\s+block_type: (\w+),/gu),
  ];
  assert.deepEqual(
    blocks.map((match) => match[3]),
    /^<script setup\b[^>]*>([\s\S]*?)<\/script>/u.exec(authored)?.[1].length
      ? ["ScriptSetup", "Template"]
      : ["Template"],
  );
  for (const [, start, end] of blocks)
    assert.ok(+start <= +end && +end <= Buffer.byteLength(authored));
}

export function validateLexicalDollarPackets(root) {
  const read = (name) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));
  const text = (name) => fs.readFileSync(path.join(root, name), "utf8");
  const { metadata, manifest } = validateMetadata(root);
  const custody = read("corpus/inputs.json");
  const expected = read("corpus/expected.json");
  for (const [file] of manifest.corpusBodies)
    assert.deepEqual(
      fs.readFileSync(path.join(root, "source", file)),
      fs.readFileSync(path.join(root, "corpus", file.split("lexical-dollar-slot-6879/")[1])),
    );
  for (const input of custody.inputs) {
    const body = fs.readFileSync(path.join(root, "corpus", input.path));
    assert.equal(body.length, input.byteLength);
    assert.equal(digest(body), input.sha256);
  }
  assert.equal(custody.inputs.length, 8);
  assert.equal(custody.nativeLaw.test, nativeLaw);
  const expectedBody = fs.readFileSync(path.join(root, "corpus/expected.json"));
  assert.equal(expectedBody.length, custody.expected.byteLength);
  assert.equal(digest(expectedBody), custody.expected.sha256);
  for (const phase of ["producer", "native"]) {
    assert.equal(text(`${phase}.status`).trim(), "0");
    const actualNames = [...text(`${phase}.stdout`).matchAll(/^test (\S+) \.\.\. ok$/gmu)].map(
      (match) => match[1],
    );
    assert.deepEqual(actualNames, phase === "producer" ? producerLaws : [nativeLaw]);
    assert.match(
      text(`${phase}.stdout`),
      phase === "producer"
        ? /test result: ok\. 6 passed; 0 failed; 0 ignored;/u
        : /test result: ok\. 1 passed; 0 failed; 0 ignored;/u,
    );
    assert.equal(text(`${phase}.stderr`), "");
    assert.deepEqual(text(`${phase}.argv.nul`).split("\0"), [
      metadata.testBinary.path,
      phase === "producer" ? "virtual_ts::scope::globals::instance::tests::" : nativeLaw,
      ...(phase === "native" ? ["--exact"] : []),
      "--nocapture",
      "--test-threads=1",
      "",
    ]);
  }
  const actual = read("native/normal-batch-results.json");
  assert.equal(actual.length, 9);
  assert.equal(expected.length, 9);
  const packets = fs
    .readdirSync(path.join(root, "native"))
    .filter((name) => name.endsWith("-native-cli.json"))
    .sort((a, b) => a.localeCompare(b));
  assert.deepEqual(
    fs.readdirSync(path.join(root, "native")).sort((a, b) => a.localeCompare(b)),
    [...packets, "normal-batch-results.json"].sort((a, b) => a.localeCompare(b)),
  );
  assert.deepEqual(
    fs.readdirSync(path.join(root, "producer")).sort((a, b) => a.localeCompare(b)),
    ["producer-scopes-false.json", "producer-scopes-true.json"],
  );
  const ownerSource = text(
    "source/crates/vize_canon/src/virtual_ts/scope/globals/instance/tests/lexical_scope.rs",
  );
  const template = /const TEMPLATE: &str = r#"([\s\S]+?)"#;/u.exec(ownerSource)[1];
  const originalSource = text(
    "source/crates/vize_canon/src/virtual_ts/scope/globals/instance/tests.rs",
  );
  const script = JSON.parse(/const SCRIPT: &str = (".*");/u.exec(originalSource)[1]);
  for (const strict of [false, true]) {
    const producer = read(`producer/producer-scopes-${strict}.json`);
    assert.equal(producer.strictInstanceGlobals, strict);
    assert.equal(producer.template, template);
    assert.equal(producer.script, script);
    assert.equal(producer.templateOffset, 1000);
    assert.ok(producer.completeSummary.length > 0);
    assert.ok(producer.expressions.length > 0 && producer.scopes.length > 0);
  }
  assert.deepEqual(
    packets,
    cases.map((_, i) => `${String(i).padStart(3, "0")}-native-cli.json`),
  );
  for (let i = 0; i < cases.length; i++) {
    const [name, sourceFile, strict] = cases[i];
    const observation = actual[i];
    const packet = read(`native/${packets[i]}`);
    assert.equal(observation.case, name);
    assert.equal(expected[i].case, name);
    assert.equal(observation.strictInstanceGlobals, strict);
    assert.deepEqual(observation.diagnostics, expected[i].diagnostics);
    assert.equal(observation.success, expected[i].success);
    assert.equal(observation.exitCode, expected[i].exitCode);
    const inputs = [
      { file: "src/Panel.vue", source: text("corpus/Panel.vue.txt") },
      { file: "src/App.vue", source: text(`corpus/${sourceFile}`) },
      { file: "tsconfig.json", source: text("corpus/case.tsconfig.json.txt") },
    ];
    assert.deepEqual(observation.inputs, inputs);
    assert.equal(packet.projectRoot, observation.projectRoot);
    assert.ok(path.isAbsolute(packet.projectRoot) && path.isAbsolute(packet.virtualRoot));
    const vuePath = observation.vueManifestPath;
    assert.equal(vuePath, metadata.vueManifest.path);
    assert.equal(Buffer.byteLength(observation.vueManifestBytes), metadata.vueManifest.byteLength);
    assert.equal(digest(observation.vueManifestBytes), metadata.vueManifest.sha256);
    assert.equal(fs.realpathSync(vuePath), vuePath);
    assert.equal(fs.readFileSync(vuePath, "utf8"), observation.vueManifestBytes);
    assert.equal(JSON.parse(observation.vueManifestBytes).name, "vue");
    assert.equal(observation.vueManifestPath, actual[0].vueManifestPath);
    assert.equal(observation.vueManifestBytes, actual[0].vueManifestBytes);
    assert.equal(packet.nativeBinary, metadata.native.path);
    assert.equal(fs.realpathSync(observation.nativeBinary), metadata.native.path);
    assert.equal(packet.nativeSha256, metadata.native.sha256);
    assert.equal(observation.nativeSha256, metadata.native.sha256);
    assert.equal(packet.route, "native-cli");
    assert.equal(packet.completion, "REAPED");
    assert.equal(packet.nativePid, null);
    assert.equal(packet.actualCwd, packet.virtualRoot);
    assert.equal(packet.requestedConfigPath, path.join(packet.virtualRoot, "tsconfig.json"));
    assert.equal(packet.authoredConfigPath, path.join(packet.projectRoot, "tsconfig.json"));
    assert.equal(packet.authoredConfigBytes, inputs[2].source);
    assert.deepEqual(packet.actualArgs, [
      "--checkers",
      "1",
      "--incremental",
      "--tsBuildInfoFile",
      path.join(packet.virtualRoot, ".vize-buildinfo", "tsconfig.tsbuildinfo"),
      "--pretty",
      "false",
      "--project",
      packet.requestedConfigPath,
    ]);
    const config = JSON.parse(packet.requestedConfigBytes);
    assert.deepEqual(config, {
      compilerOptions: {
        ...JSON.parse(inputs[2].source).compilerOptions,
        allowImportingTsExtensions: true,
        noUncheckedSideEffectImports: false,
      },
      exclude: [],
      include: [
        "__vize_helpers.d.ts",
        "__vize_vue_modules.d.ts",
        "src/App.vue.ts",
        "src/Panel.vue.ts",
      ],
    });
    assert.equal(packet.exitCode, observation.exitCode);
    assert.equal(packet.success, observation.success);
    assert.equal(packet.status, `exit status: ${packet.exitCode}`);
    assert.deepEqual(bytes(packet.rawStderrBytes), Buffer.alloc(0));
    const stdout = utf8(bytes(packet.rawStdoutBytes));
    assert.equal(stdout.length === 0, packet.exitCode === 0);
    assert.deepEqual(
      [...stdout.matchAll(/^.*\(\d+,\d+\): error TS(\d+):/gmu)].map((match) => Number(match[1])),
      observation.diagnostics.map((diagnostic) => diagnostic.code),
    );
    assert.deepEqual(
      packet.generatedFiles.map((file) => file.original),
      ["src/App.vue", "src/Panel.vue"].map((file) => path.join(packet.projectRoot, file)),
    );
    for (const generated of packet.generatedFiles) {
      const input = inputs.find(
        (entry) => path.join(packet.projectRoot, entry.file) === generated.original,
      );
      assert.equal(generated.authoredSource, input.source);
      assert.equal(generated.virtual, path.join(packet.virtualRoot, `${input.file}.ts`));
      assert.ok(config.include.includes(`${input.file}.ts`));
      assert.equal(typeof generated.code, "string");
      assert.ok(generated.code.length > 0);
      completeMap(generated.sourceMap, generated.code, generated.authoredSource);
    }
    assert.deepEqual(packet.unobserved, [
      "native PID",
      "native engine normalized options",
      "native engine loaded source-file vector",
    ]);
  }
  return {
    source: metadata.source,
    nativeCases: 9,
    producerLaws: 6,
    ignored: 0,
    apiLspCredit: false,
    original400Credit: false,
    performanceCredit: false,
  };
}

if (
  process.argv[1] &&
  fs.existsSync(process.argv[1]) &&
  fs.realpathSync(process.argv[1]) === fs.realpathSync(new URL(import.meta.url))
)
  fs.writeFileSync(
    path.join(process.argv[2], "acceptance.json"),
    `${JSON.stringify(validateLexicalDollarPackets(process.argv[2]), null, 2)}\n`,
  );
