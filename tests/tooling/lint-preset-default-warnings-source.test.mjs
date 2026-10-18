import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (name) => fs.readFileSync(path.join(root, name));
const sha = (body) => createHash("sha256").update(body).digest("hex");
const directory = "tests/_fixtures/lint-preset-default-warnings";
const bytes = read(`${directory}/successor.json`);
assert.equal(sha(bytes), "30d630da3f38e9eb667e2097bda2b65d8e0e3b82bb5f62c61e4aead8a4cdc295");
const manifest = JSON.parse(bytes);

// #9357 (07f5645623a8d26d8909a9a53f8636682ce2192c) removed only
// retired extension-host scheduling. Keep the original preset audit frozen;
// restore these two exact source hunks for that audit, never for execution.
const retiredExtensionHost = {
  path: ".config/nextest.toml",
  before: {
    bytes: 5213,
    sha256: "d7aca4d0556a5ab2a4ef81c16d022883aaf2e17ddc48de3235e9cbe028596e86",
  },
  after: {
    bytes: 4801,
    sha256: "ff231a2a36a5b1a2fafa247a79785fb7aa08b70b035a3f14392a5648880d1595",
  },
  hunks: [
    {
      before:
        "[test-groups]\n# Guest builders share target/contract-guests. Their OnceLocks are process-local,\n# so nextest must limit the guest builds across test processes on each runner.\ncontract-guests = { max-threads = 1 }\n\n[[profile.pr.overrides]]\nfilter = 'package(=vize_extension_host)'\ntest-group = \"contract-guests\"\n\n# Full T1 has no default-filter: every ordinary workspace case remains selected.\n",
      after: "# Full T1 has no default-filter: every ordinary workspace case remains selected.\n",
    },
    {
      before:
        '[profile.full.junit]\npath = "junit.xml"\nstore-success-output = false\nstore-failure-output = true\n\n[[profile.full.overrides]]\nfilter = \'package(=vize_extension_host)\'\ntest-group = "contract-guests"\n',
      after:
        '[profile.full.junit]\npath = "junit.xml"\nstore-success-output = false\nstore-failure-output = true\n',
    },
  ],
};

// Two later, exact receipt-retention edits leave every original preset proof frozen.
const presetCaseName = {
  path: retiredExtensionHost.path,
  before: retiredExtensionHost.after,
  after: {
    bytes: 4794,
    sha256: "a15ff1ec341a5eae7a3267c286834d2641a685d269ac44dd75c919995c0e19c5",
  },
  hunks: [
    {
      before:
        "# Retain the 38 exact default/off/repair/alias CLI packets without changing selection.\n[[profile.default.overrides]]\nfilter = 'package(=vize) & binary(=lint_preset_scope_cli) & test(=six_default_presets_preserve_38_complete_cli_warning_off_repair_and_alias_packets)'\n",
      after:
        "# Retain complete default/off/repair/alias CLI packets without changing selection.\n[[profile.default.overrides]]\nfilter = 'package(=vize) & binary(=lint_preset_scope_cli) & test(=six_default_presets_preserve_complete_cli_warning_off_repair_and_alias_packets)'\n",
    },
  ],
};
const callbackReceipts = {
  path: retiredExtensionHost.path,
  before: presetCaseName.after,
  after: {
    bytes: 4884,
    sha256: "b72ab346f8286e68abe2d549de7c32cebc131fd61e122ef327f1e8329abd0ea0",
  },
  hunks: [
    {
      before:
        "  test(=script_setup_declaration_forms_preserve_complete_cli_packets_and_unqualified_this_frontier)\n",
      after:
        "  test(=script_setup_declaration_forms_preserve_complete_cli_packets_and_unqualified_this_frontier) |\n  test(=legacy_instance_cli::callback_return_ownership_keeps_all_six_whole_cli_packets)\n",
    },
  ],
};

function identity(body, expected) {
  assert.equal(body.length, expected.bytes);
  assert.equal(sha(body), expected.sha256);
}

// This inverse is an audit of these exact edits. It is never installed as an
// adapter for an old source, compiled, runtime or performance acceptance gate.
function inverse(row, body) {
  identity(body, row.after);
  let text = body.toString("utf8");
  for (const hunk of row.hunks) {
    assert.notEqual(hunk.before, hunk.after);
    assert.equal(text.split(hunk.after).length, 2, row.path);
    text = text.replace(hunk.after, hunk.before);
  }
  const before = Buffer.from(text);
  identity(before, row.before);
  return before;
}

// #9399 adds exactly two authored cases and two Essential memberships. Preserve
// the old source/fixture audit as a historical stage, not a runtime adapter.
const essentialRules = [
  "vue/no-non-component-keep-alive-child",
  "script/no-duplicate-attr-inheritance",
];
const essentialCases = ["essential-keep-alive", "essential-attrs-forwarding"];
const jsonBytes = (value) => Buffer.from(JSON.stringify(value, null, 2) + "\n");
function canonical(body) {
  const value = JSON.parse(body.toString());
  assert.deepEqual(body, jsonBytes(value)); // Never discard unrelated whitespace/edits.
  return value;
}
function originalEssentialSource(row, body) {
  if (row.path === "crates/vize_patina/src/linter/script_rules/registry/rules.rs") {
    const text = body.toString();
    const entries = text
      .split("\n")
      .filter((line) => line.includes("rule_name: RULE_NO_DUPLICATE_ATTR_INHERITANCE,"));
    assert.equal(entries.length, 1);
    const entry = entries[0];
    assert.equal(entry.split("presets: ESSENTIAL_SCRIPT_PRESETS").length, 2);
    return Buffer.from(
      text.replace(
        entry,
        entry.replace("presets: ESSENTIAL_SCRIPT_PRESETS", "presets: HAPPY_PATH_SCRIPT_PRESETS"),
      ),
    );
  }
  if (row.path === "tests/_fixtures/patina-eslint-vue-rule-map.json") {
    const value = canonical(body);
    const presets = value.entries["vue/no-duplicate-attr-inheritance"].patinaPresets;
    assert.equal(presets.filter((preset) => preset === "essential").length, 1);
    presets.splice(presets.indexOf("essential"), 1);
    return jsonBytes(value);
  }
  return body;
}
function originalEssentialMembership(body) {
  const text = body.toString();
  const at = text.indexOf("{");
  const table = canonical(Buffer.from(text.slice(at)));
  for (const rule of essentialRules) {
    assert.equal(table.essential.filter((name) => name === rule).length, 1);
    table.essential.splice(table.essential.indexOf(rule), 1);
  }
  return Buffer.concat([Buffer.from(text.slice(0, at)), jsonBytes(table)]);
}
function currentCases(body) {
  // This is the reviewed 81f5033 additive fixture, independently exercised by
  // all 76 complete CLI receipts in native queue Check 39043300642, not a new capture.
  identity(body, {
    bytes: 12915,
    sha256: "d347790320c83bbfb0ae513de8781559312766f48ac5c07483ce753146d579ea",
  });
  const value = JSON.parse(body);
  assert.deepEqual(
    value.cases.map(({ id }) => id),
    ["else-for", "deep-props", ...essentialCases],
  );
  const { essentialExtensionAuthority, essentialBaselineAuthority, ...original } = value;
  assert(essentialExtensionAuthority && essentialBaselineAuthority);
  original.cases = value.cases.slice(0, 2);
  identity(jsonBytes(original), manifest.cases); // Keep the original 5090-byte oracle.
  for (const [index, sha256] of [
    "c99b63e767e42a975eb39bdfe14bb25b9fa2328165b0d29d49a9a27d9aec57b8",
    "7588faadaa8522d8fdc7783193decbe0ff3fcb611d183d08cb371e5f7af02399",
  ].entries())
    assert.equal(sha(jsonBytes(value.cases[index + 2])), sha256);
  return value;
}

function inverseCurrent(row, body) {
  return inverse(
    row,
    row.path === retiredExtensionHost.path
      ? inverse(retiredExtensionHost, inverse(presetCaseName, inverse(callbackReceipts, body)))
      : originalEssentialSource(row, body),
  );
}

void test("callback retention and preset case naming have only their exact finite inverses", () => {
  let body = read(callbackReceipts.path);
  for (const row of [callbackReceipts, presetCaseName]) {
    const text = body.toString();
    for (const hunk of row.hunks)
      for (const changed of [
        text.replace(hunk.after, hunk.before),
        text.replace(hunk.after, hunk.after + hunk.after),
        text + "\n# unrelated edit\n",
      ])
        assert.throws(() => inverse(row, Buffer.from(changed)));
    body = inverse(row, body);
  }
  identity(body, retiredExtensionHost.after);
});

function inverseMembership(body) {
  body = originalEssentialMembership(body);
  identity(body, manifest.membership.after);
  const text = body.toString("utf8");
  const at = text.indexOf("{");
  const table = JSON.parse(text.slice(at));
  for (const preset of ["happy_path", "ecosystem", "opinionated", "nuxt"]) {
    const values = table[preset];
    const rule = "vue/no-use-v-else-with-v-for";
    assert.equal(values.filter((name) => name === rule).length, 1);
    assert.equal(values[values.indexOf("vue/no-use-v-if-with-v-for") + 1], rule);
    values.splice(values.indexOf(rule), 1);
    if (preset === "opinionated" || preset === "nuxt") {
      values.splice(values.indexOf("vue/no-array-index-key") + 1, 0, rule);
      const deep = "script/no-deep-destructure-in-props";
      assert.equal(values.filter((name) => name === deep).length, 1);
      values.splice(values.indexOf(deep), 1);
    }
  }
  const before = Buffer.from(text.slice(0, at) + JSON.stringify(table, null, 2) + "\n");
  identity(before, manifest.membership.before);
  return table;
}

void test("only the exact four-preset registration and metadata edits invert to the unchanged parent", () => {
  assert.equal(manifest.schema, "vize.lint.preset-warning-source-successor");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.parent, "883b800148b0faca3f4590d0b029f86e494a0b5e");
  assert.deepEqual(manifest.credit, { compiled: false, native: false, performance: false });
  assert.deepEqual(
    manifest.sources.map(({ path: name }) => name),
    [
      "crates/vize_patina/src/rule/happy_path.rs",
      "crates/vize_patina/src/rules/opinionated/vue.rs",
      "crates/vize_patina/src/linter/script_rules/registry.rs",
      "crates/vize_patina/src/linter/script_rules/registry/rules.rs",
      "crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs",
      "crates/vize_patina/src/preset.rs",
      ".config/nextest.toml",
      "tests/_fixtures/patina-eslint-vue-rule-map.json",
    ],
  );
  assert.deepEqual(
    manifest.sources.find((row) => row.path === retiredExtensionHost.path).after,
    retiredExtensionHost.before,
  );
  for (const row of manifest.sources) inverseCurrent(row, read(row.path));
  const old = inverseMembership(read(manifest.membership.path));
  const table = JSON.parse(read(manifest.membership.path).toString().split("---\n").at(-1));
  assert.deepEqual(
    Object.fromEntries(Object.entries(table).map(([key, value]) => [key, value.length])),
    {
      ecosystem: 148,
      essential: 79,
      happy_path: 140,
      incremental: 0,
      nuxt: 197,
      opinionated: 198,
      opt_in: 104,
    },
  );
  assert.deepEqual(
    table.essential.filter((name) => !essentialRules.includes(name)),
    old.essential,
  );
  assert.deepEqual(table.incremental, []);
  assert.deepEqual(table.opt_in, old.opt_in);
  for (const source of manifest.preservedCorpora) identity(read(source.path), source);
  identity(read(manifest.unchangedRule.path), manifest.unchangedRule);
});

void test("missing, duplicated and unrelated source edits cannot receive the finite inverse", () => {
  for (const row of manifest.sources) {
    const body =
      row.path === retiredExtensionHost.path
        ? inverse(
            retiredExtensionHost,
            inverse(presetCaseName, inverse(callbackReceipts, read(row.path))),
          )
        : originalEssentialSource(row, read(row.path));
    const hunk = row.hunks[0];
    for (const text of [
      body.toString().replace(hunk.after, hunk.before),
      body.toString().replace(hunk.after, hunk.after + hunk.after),
      body.toString() + "\n// unrelated edit\n",
    ])
      assert.throws(() => inverse(row, Buffer.from(text)), row.path);
  }
  const retired = inverse(
    presetCaseName,
    inverse(callbackReceipts, read(retiredExtensionHost.path)),
  ).toString();
  for (const hunk of retiredExtensionHost.hunks)
    for (const text of [
      retired.replace(hunk.after, hunk.before),
      retired.replace(hunk.after, hunk.after + hunk.after),
      retired + "\n# unrelated edit\n",
    ])
      assert.throws(() => inverse(retiredExtensionHost, Buffer.from(text)));
  const body = read(manifest.membership.path);
  assert.throws(() =>
    inverseMembership(Buffer.from(body.toString().replace('"essential": [', '"unknown": ['))),
  );
  assert.throws(() => inverseMembership(Buffer.concat([body, Buffer.from("\n")])));
});

void test("Essential additions reject missing, duplicate and unrelated policy or oracle changes", () => {
  const registry = manifest.sources.find((row) => row.path.endsWith("registry/rules.rs"));
  const map = manifest.sources.find(
    (row) => row.path === "tests/_fixtures/patina-eslint-vue-rule-map.json",
  );
  const text = read(registry.path).toString();
  const entry = text
    .split("\n")
    .find((line) => line.includes("rule_name: RULE_NO_DUPLICATE_ATTR_INHERITANCE,"));
  for (const changed of [
    text.replace(entry, entry.replace("ESSENTIAL_SCRIPT_PRESETS", "HAPPY_PATH_SCRIPT_PRESETS")),
    text.replace(entry, entry + "\n" + entry),
    text + "\n// unrelated edit\n",
  ])
    assert.throws(() => inverseCurrent(registry, Buffer.from(changed)));
  for (const change of [
    (value) => {
      const presets = value.entries["vue/no-duplicate-attr-inheritance"].patinaPresets;
      presets.splice(presets.indexOf("essential"), 1);
    },
    (value) => value.entries["vue/no-duplicate-attr-inheritance"].patinaPresets.push("essential"),
    (value) => {
      value.unrelated = true;
    },
  ]) {
    const value = JSON.parse(read(map.path));
    change(value);
    assert.throws(() => inverseCurrent(map, jsonBytes(value)));
  }
  for (const change of [
    (value) => value.cases.pop(),
    (value) => value.cases.push(value.cases[2]),
    (value) => {
      value.cases[0].diagnostic.start++;
    },
    (value) => {
      value.cases[3].cliMessage.help = value.cases[3].diagnostic.help;
    },
  ]) {
    const value = JSON.parse(read(manifest.cases.path));
    change(value);
    assert.throws(() => currentCases(jsonBytes(value)));
  }
  const membership = read(manifest.membership.path).toString();
  const at = membership.indexOf("{");
  for (const change of [
    (table) => table.essential.splice(table.essential.indexOf(essentialRules[0]), 1),
    (table) => table.essential.push(essentialRules[1]),
    (table) => table.essential.push("unrelated/rule"),
  ]) {
    const table = JSON.parse(membership.slice(at));
    change(table);
    assert.throws(() =>
      inverseMembership(Buffer.concat([Buffer.from(membership.slice(0, at)), jsonBytes(table)])),
    );
  }
});

void test("independently authored native and CLI packets preserve complete warning and source ranges", () => {
  const body = read(manifest.cases.path);
  const corpus = currentCases(body);
  assert.equal(corpus.cases.length, 4);
  for (const entry of corpus.cases) {
    assert.equal(sha(Buffer.from(entry.source)), entry.sourceSha256);
    const d = entry.diagnostic;
    const c = entry.cliMessage;
    assert.equal(d.severity, "warning");
    assert.equal(c.severity, 1);
    assert.equal(c.ruleId, d.rule_name);
    assert.equal(c.message, `[vize:${d.rule_name}] ${d.message}`);
    assert.equal(
      c.help,
      entry.id === "essential-attrs-forwarding"
        ? "Opt out of the automatic inheritance so only this v-bind applies them, e.g. defineOptions({ inheritAttrs: false }) or inheritAttrs: false in the component options."
        : d.help,
    );
    assert.equal(d.fix, null);
    assert.deepEqual(
      d.labels,
      entry.id === "essential-attrs-forwarding"
        ? [{ message: "attributes applied twice", start: 53, end: 59 }]
        : [],
    );
    if (entry.id === "essential-attrs-forwarding")
      assert.equal(Buffer.from(entry.source).subarray(53, 59).toString(), "$attrs");
    for (const [offset, line, column] of [
      [d.start, c.line, c.column],
      [d.end, c.endLine, c.endColumn],
    ]) {
      const prefix = Buffer.from(entry.source).subarray(0, offset).toString("utf8");
      assert.equal(prefix.split("\n").length, line);
      assert.equal(Array.from(prefix.split("\n").at(-1) ?? "").length + 1, column);
    }
    assert.equal(entry.enabledPresets.includes("essential"), essentialCases.includes(entry.id));
    assert(!entry.enabledPresets.includes("incremental"));
    assert.notEqual(entry.source, entry.repair);
  }
});

void test("the existing warning baseline preserves every original source, repair and promoted descriptor", () => {
  const before = read(manifest.authoredCasesBeforeBaseline.path);
  identity(before, manifest.authoredCasesBeforeBaseline);
  assert.equal(sha(before), "731aa0c0dbeb7b6bf9f046dbc8d58b36fb00b89eb7317e9b89ffc8521c7959d7");
  const original = JSON.parse(before);
  const current = currentCases(read(manifest.cases.path));
  assert.equal(current.cases.length, original.cases.length + essentialCases.length);
  for (const [index, row] of original.cases.entries()) {
    const actual = current.cases[index];
    for (const key of Object.keys(row))
      assert.deepEqual(actual[key], row[key], `${row.id} / ${key}`);
    assert.deepEqual(
      Object.keys(actual).filter((key) => !Object.hasOwn(row, key)),
      ["baselinePresets", "baselineDiagnostics", "baselineCliMessages"],
    );
  }
  assert.equal(current.baselineAuthority.run, 38991839496);
  assert.equal(current.baselineAuthority.job, 117275806128);
  assert.equal(
    current.baselineAuthority.rawLogSha256,
    "70465d7e2aabd3bed3c1ef186ddab3fec829061fc2f63078c2891f2c91d8d31f",
  );
});

void test("all generated locale tables and bilingual references expose the actual preset memberships", () => {
  const locales = ["en", "ja", "fr", "pt-BR", "zh-CN"];
  const presets = (line) =>
    [...line.matchAll(/`([^`]+)`/g)]
      .map((match) => match[1])
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  for (const { name, expected, section } of [
    {
      name: "vue/no-use-v-else-with-v-for",
      expected: ["ecosystem", "happy-path", "nuxt", "opinionated"],
      section: "vue",
    },
    {
      name: "script/no-deep-destructure-in-props",
      expected: ["nuxt", "opinionated"],
      section: "type-and-script",
    },
  ]) {
    const slug = name.replaceAll("/", "-");
    for (const filename of locales.map(
      (locale) => `docs/content/generated/rules/${locale}/${section}.md`,
    )) {
      const body = read(filename).toString();
      const block = body.split(`### \`${name}\`\n`)[1]?.split("\n### ")[0];
      assert(block, filename);
      const line = block
        .split("\n")
        .find((line) => /^(?:Presets|プリセット|Préréglages|Predefinições|预设):/.test(line));
      assert(line, filename);
      assert.deepEqual(presets(line), expected, filename);
    }
    for (const locale of ["", "ja/"]) {
      const filename = `docs/content/${locale}rules/reference/${slug}.md`;
      const line = read(filename)
        .toString()
        .split("\n")
        .find((line) => /^(?:Presets|プリセット):/.test(line));
      assert(line, filename);
      assert.deepEqual(presets(line), expected, filename);
    }
    for (const filename of [
      "docs/content/rules/all.md",
      "docs/content/ja/rules/all.md",
      ...["en", "ja"].map((locale) => `docs/content/generated/rules/${locale}/all.md`),
    ]) {
      const line = read(filename)
        .toString()
        .split("\n")
        .find((line) => line.startsWith(`| [\`${name}\`]`));
      assert(line, filename);
      const cells = line.split(" | ");
      const severity = cells.indexOf("`warning`");
      assert(severity >= 0, filename);
      assert.deepEqual(presets(cells[severity + 1]), expected, filename);
    }
  }
});
