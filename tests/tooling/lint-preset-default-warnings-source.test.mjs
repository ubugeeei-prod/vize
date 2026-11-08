import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  canonical,
  createClassSourceInverse,
  createSourceReader,
  identity,
  inverse,
  inverseCurrentSource,
  jsonBytes,
  loadCurrentSourceSuccessors,
  replaceBytes,
  sha,
} from "./support/lint-preset-source-successors.ts";
import {
  currentPresetCases,
  inversePatinaCurrentSource,
} from "./support/lint-preset-latest-default-expansion.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directory = "tests/_fixtures/lint-preset-default-warnings";

// Later preset-default changes are finite successor rows, newest last. Each
// row pins both byte identities and is undone before the audits below, so they
// keep judging the tree they reviewed while presets grow.
const presetSuccessorBytes = fs.readFileSync(path.join(root, directory, "preset-successors.json"));
const presetSuccessors = JSON.parse(presetSuccessorBytes);
const currentSources = loadCurrentSourceSuccessors(
  root,
  directory,
  presetSuccessorBytes,
  presetSuccessors,
);
// The audit judges the nextest.toml it reviewed; CI keeps editing the live file.
const liveNextest = ".config/nextest.toml";
const reviewedNextest = `${directory}/reviewed-nextest.toml`;
const read = createSourceReader({
  root,
  liveNextest,
  reviewedNextest,
  currentSources,
  presetSuccessors,
  latestSourceInverse: inversePatinaCurrentSource,
});
const bytes = read(`${directory}/successor.json`);
assert.equal(sha(bytes), "30d630da3f38e9eb667e2097bda2b65d8e0e3b82bb5f62c61e4aead8a4cdc295");
const manifest = JSON.parse(bytes);

// Keep the original retirement/receipt edits and this additive class-default
// successor pinned as finite source audits, never runtime acceptance adapters.
const retentionBytes = read(`${directory}/source-retention-successors.json`);
assert.equal(
  sha(retentionBytes),
  "3f9a60c0a76c5e6ed8cddf9aa42654f584fca734dcdad4b5780303abce378c74",
);
const {
  retiredExtensionHost,
  presetCaseName,
  callbackReceipts,
  staticClassRegistration,
  staticClassMembership,
} = JSON.parse(retentionBytes);

// Later additions are source-only history. Both byte directions must recover
// complete authenticated bodies before the original finite audits run.

// This inverse is an audit of these exact edits. It is never installed as an
// adapter for an old source, compiled, runtime or performance acceptance gate.

// Restore only the reviewed class-default edits for the frozen earlier audit.
const originalStaticClassSource = createClassSourceInverse(
  staticClassRegistration,
  staticClassMembership,
);

// #9399 adds exactly two authored cases and two Essential memberships. Preserve
// the old source/fixture audit as a historical stage, not a runtime adapter.
const essentialRules = [
  "vue/no-non-component-keep-alive-child",
  "script/no-duplicate-attr-inheritance",
];
const essentialCases = ["essential-keep-alive", "essential-attrs-forwarding"];
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
  return currentPresetCases(body, essentialCases, manifest);
}

function inverseCurrent(row, body) {
  return inverse(
    row,
    row.path === retiredExtensionHost.path
      ? inverse(retiredExtensionHost, inverse(presetCaseName, inverse(callbackReceipts, body)))
      : originalEssentialSource(row, originalStaticClassSource(row.path, body)),
  );
}

void test("current source successors preserve whole histories and reject missing, partial, duplicate and unrelated bytes", () => {
  for (const row of currentSources.sources) {
    const current = inversePatinaCurrentSource(
      row.path,
      fs.readFileSync(path.join(root, row.path)),
    );
    identity(inverseCurrentSource(row, current), row.before);
    const hunk = row.hunks.find((hunk) => hunk.after.length > 0);
    const changedByte = Buffer.from(current);
    changedByte[0] ^= 1;
    for (const changed of [
      null,
      current.subarray(0, current.length - 1),
      Buffer.concat([Buffer.from("// unrelated prefix\n"), current]),
      Buffer.concat([current, Buffer.from("\n// unrelated suffix\n")]),
      replaceBytes(current, hunk.afterOffset, Buffer.from(hunk.after), Buffer.from(hunk.before)),
      replaceBytes(
        current,
        hunk.afterOffset,
        Buffer.from(hunk.after),
        Buffer.from(hunk.after + hunk.after),
      ),
      changedByte,
    ]) {
      assert.notDeepEqual(changed, current);
      assert.throws(() => inverseCurrentSource(row, changed));
      if (changed)
        assert.throws(() =>
          inverseCurrentSource(
            { ...row, after: { bytes: changed.length, sha256: sha(changed) } },
            changed,
          ),
        );
    }
    const shifted = structuredClone(row);
    shifted.hunks[0].afterOffset++;
    assert.throws(() => inverseCurrentSource(shifted, current));
  }
});

void test("the latest general-default expansion has one exact finite inverse", () => {
  for (const row of currentSources.sources) {
    const live = fs.readFileSync(path.join(root, row.path));
    const predecessor = inversePatinaCurrentSource(row.path, live);
    identity(predecessor, row.after);
    if (predecessor.equals(live)) continue;
    for (const changed of [
      live.subarray(0, live.length - 1),
      Buffer.concat([live, Buffer.from("\n")]),
      Buffer.concat([Buffer.from("// unrelated\n"), live]),
    ])
      assert.throws(() => identity(inverseLatestDefaultExpansion(row.path, changed), row.after));
  }
});

void test("class-default successors reject missing, duplicate and unrelated edits", () => {
  for (const row of [staticClassRegistration, staticClassMembership]) {
    const body = read(row.path);
    identity(originalStaticClassSource(row.path, body), row.before);
    const needle = row.hunks?.[0].after ?? '    "html/no-duplicate-class",\n';
    const text = body.toString();
    assert(text.includes(needle));
    for (const changed of [
      text.replace(needle, ""),
      text.replace(needle, needle + needle),
      text + "\n",
    ])
      assert.throws(() => originalStaticClassSource(row.path, Buffer.from(changed)));
  }
});

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
  body = originalEssentialMembership(originalStaticClassSource(staticClassMembership.path, body));
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
      ecosystem: 149,
      essential: 79,
      happy_path: 141,
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
        : originalEssentialSource(row, originalStaticClassSource(row.path, read(row.path)));
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

void test("every locale's rule page and index expose the actual preset memberships", () => {
  const locales = ["", "ja/", "fr/", "pt-BR/", "zh-CN/"];
  const presets = (line) =>
    [...line.matchAll(/`([^`]+)`/g)]
      .map((match) => match[1])
      .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  for (const { name, expected, smallest } of [
    {
      name: "vue/no-use-v-else-with-v-for",
      expected: ["ecosystem", "happy-path", "nuxt", "opinionated"],
      smallest: ["happy-path"],
    },
    {
      name: "script/no-deep-destructure-in-props",
      expected: ["nuxt", "opinionated"],
      smallest: ["nuxt", "opinionated"],
    },
  ]) {
    const slug = name.replaceAll("/", "-");
    for (const locale of locales) {
      const filename = `docs/content/${locale}rules/reference/${slug}.md`;
      // The facts row: severity, presets, automatic fix, options.
      const line = read(filename)
        .toString()
        .split("\n")
        .find((line) => line.startsWith("| `warning` | "));
      assert(line, filename);
      assert.deepEqual(presets(line.split(" | ")[1]), expected, filename);
      const index = `docs/content/${locale}rules/all.md`;
      const row = read(index)
        .toString()
        .split("\n")
        .find((line) => line.includes(`[\`${name}\`](./reference/${slug}.md)`));
      assert(row, index);
      assert.deepEqual(presets(row.split(" | ")[2]), smallest, index);
    }
  }
});

void test("the live nextest config keeps the reviewed lint preset packet overrides", () => {
  const live = fs.readFileSync(path.join(root, liveNextest), "utf8");
  for (const name of [
    "six_default_presets_preserve_complete_cli_warning_off_repair_and_alias_packets",
    "callback_return_ownership_keeps_all_six_whole_cli_packets",
  ])
    assert.ok(live.includes(name), name);
});
