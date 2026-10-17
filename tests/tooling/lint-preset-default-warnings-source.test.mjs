import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse as parseToml, stringify as stringifyToml } from "@iarna/toml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (name) => fs.readFileSync(path.join(root, name));
const sha = (body) => createHash("sha256").update(body).digest("hex");
const directory = "tests/_fixtures/lint-preset-default-warnings";
const bytes = read(`${directory}/successor.json`);
assert.equal(sha(bytes), "30d630da3f38e9eb667e2097bda2b65d8e0e3b82bb5f62c61e4aead8a4cdc295");
const manifest = JSON.parse(bytes);
// The scheduler has evolved since this source-history audit. Preserve its
// authentic old bytes; the live output-retention override is checked below.
const auditSource = (row) =>
  read(row.path === ".config/nextest.toml" ? `${directory}/nextest-8df6e1.toml` : row.path);
const warningFilter =
  "package(=vize) & binary(=lint_preset_scope_cli) & test(=six_default_presets_preserve_38_complete_cli_warning_off_repair_and_alias_packets)";

function assertWarningRetention(source) {
  const overrides = parseToml(source).profile?.default?.overrides ?? [];
  assert.deepEqual(
    overrides.filter((row) => row.filter === warningFilter),
    [{ filter: warningFilter, junit: { "store-success-output": true } }],
  );
}

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

function inverseMembership(body) {
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
  for (const row of manifest.sources) inverse(row, auditSource(row));
  const old = inverseMembership(read(manifest.membership.path));
  const table = JSON.parse(read(manifest.membership.path).toString().split("---\n").at(-1));
  assert.deepEqual(
    Object.fromEntries(Object.entries(table).map(([key, value]) => [key, value.length])),
    {
      ecosystem: 148,
      essential: 77,
      happy_path: 140,
      incremental: 0,
      nuxt: 197,
      opinionated: 198,
      opt_in: 104,
    },
  );
  assert.deepEqual(table.essential, old.essential);
  assert.deepEqual(table.incremental, []);
  assert.deepEqual(table.opt_in, old.opt_in);
  for (const source of manifest.preservedCorpora) identity(read(source.path), source);
  identity(read(manifest.unchangedRule.path), manifest.unchangedRule);
});

void test("missing, duplicated and unrelated source edits cannot receive the finite inverse", () => {
  for (const row of manifest.sources) {
    const body = auditSource(row);
    const hunk = row.hunks[0];
    for (const text of [
      body.toString().replace(hunk.after, hunk.before),
      body.toString().replace(hunk.after, hunk.after + hunk.after),
      body.toString() + "\n// unrelated edit\n",
    ])
      assert.throws(() => inverse(row, Buffer.from(text)), row.path);
  }
  const body = read(manifest.membership.path);
  assert.throws(() =>
    inverseMembership(Buffer.from(body.toString().replace('"essential": [', '"unknown": ['))),
  );
  assert.throws(() => inverseMembership(Buffer.concat([body, Buffer.from("\n")])));
});

void test("the live nextest configuration retains the complete warning CLI output", () => {
  assertWarningRetention(read(".config/nextest.toml").toString("utf8"));
});

void test("live warning retention rejects missing, duplicate, changed-filter and disabled overrides", () => {
  const original = parseToml(read(".config/nextest.toml").toString("utf8"));
  const index = original.profile.default.overrides.findIndex((row) => row.filter === warningFilter);
  assert(index >= 0);
  for (const mutate of [
    (config, rows) => rows.splice(index, 1),
    (config, rows) => rows.push(structuredClone(rows[index])),
    (config, rows) => {
      rows[index].filter = "package(=vize)";
    },
    (config, rows) => {
      rows[index].filter = warningFilter.replace("=vize)", "=other)");
    },
    (config, rows) => {
      rows[index].filter = warningFilter.replace("=lint_preset_scope_cli", "=other");
    },
    (config, rows) => {
      rows[index].filter = warningFilter.replace("six_default_presets", "other");
    },
    (config, rows) => {
      rows[index].junit["store-success-output"] = false;
    },
    (config, rows) => {
      rows[index].junit["store-success-output"] = "true";
    },
    (config, rows) => {
      delete rows[index].junit["store-success-output"];
    },
    (config, rows) => {
      config.profile.pr.overrides.push(...rows.splice(index, 1));
    },
  ]) {
    const config = structuredClone(original);
    mutate(config, config.profile.default.overrides);
    assert.throws(() => assertWarningRetention(stringifyToml(config)));
  }
});

void test("independently authored native and CLI packets preserve complete warning and source ranges", () => {
  const body = read(manifest.cases.path);
  identity(body, manifest.cases);
  const corpus = JSON.parse(body);
  assert.equal(corpus.cases.length, 2);
  for (const entry of corpus.cases) {
    assert.equal(sha(Buffer.from(entry.source)), entry.sourceSha256);
    const d = entry.diagnostic;
    const c = entry.cliMessage;
    assert.equal(d.severity, "warning");
    assert.equal(c.severity, 1);
    assert.equal(c.ruleId, d.rule_name);
    assert.equal(c.message, `[vize:${d.rule_name}] ${d.message}`);
    assert.equal(c.help, d.help);
    assert.equal(d.fix, null);
    assert.deepEqual(d.labels, []);
    for (const [offset, line, column] of [
      [d.start, c.line, c.column],
      [d.end, c.endLine, c.endColumn],
    ]) {
      const prefix = Buffer.from(entry.source).subarray(0, offset).toString("utf8");
      assert.equal(prefix.split("\n").length, line);
      assert.equal(Array.from(prefix.split("\n").at(-1) ?? "").length + 1, column);
    }
    assert(!entry.enabledPresets.includes("essential"));
    assert(!entry.enabledPresets.includes("incremental"));
    assert.notEqual(entry.source, entry.repair);
  }
});

void test("the existing warning baseline preserves every original source, repair and promoted descriptor", () => {
  const before = read(manifest.authoredCasesBeforeBaseline.path);
  identity(before, manifest.authoredCasesBeforeBaseline);
  assert.equal(sha(before), "731aa0c0dbeb7b6bf9f046dbc8d58b36fb00b89eb7317e9b89ffc8521c7959d7");
  const original = JSON.parse(before);
  const current = JSON.parse(read(manifest.cases.path));
  assert.equal(current.cases.length, original.cases.length);
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
