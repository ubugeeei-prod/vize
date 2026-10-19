import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  baselineConfig,
  resolveBaselineRuntime,
  retainEnabledFindings,
  runBaseline,
  selectComparableRules,
} from "../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";
import { collectBaselineFindings } from "../../tools/support/compat/fixtures/lint-divergence-records.ts";
import {
  assertBudgetsPassed,
  evaluateBudget,
} from "../../tools/support/compat/fixtures/lint-divergence-budget.ts";
import { baselineScript as embedded } from "../../tools/support/fixtures/lint_divergence/runtime.ts";

const root = path.resolve(import.meta.dirname, "../..");
const digest = (body) => createHash("sha256").update(body).digest("hex");
const requireFromBench = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
const runtime = resolveBaselineRuntime();
const providers = Object.fromEntries(
  ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map((name) => {
    const manifest = requireFromBench.resolve(`${name}/package.json`);
    const entrypoint = requireFromBench.resolve(name);
    return [
      name,
      {
        version: requireFromBench(`${name}/package.json`).version,
        manifest,
        manifestSha256: digest(fs.readFileSync(manifest)),
        entrypoint,
        entrypointSha256: digest(fs.readFileSync(entrypoint)),
      },
    ];
  }),
);
const ruleMap = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json"), "utf8"),
);
const rules = selectComparableRules(ruleMap, "ecosystem", { includeUnimplemented: true }).rules;
const defaults = selectComparableRules(ruleMap, "ecosystem").rules;
const base = runtime.plugin.configs["flat/base"].find(
  (config) => config.name === "vue/base/setup-for-vue",
);
assert.equal(runtime.version, "10.9.2");
assert.equal(base.processor, "vue/vue");
assert.equal(runtime.plugin.processors.vue, runtime.plugin.processors[".vue"]);
const producer = fs.readFileSync(
  path.join(root, "tools/support/fixtures/lint_divergence/runtime.ts"),
  "utf8",
);
assert.ok(embedded, "exercise the actual TypeScript producer's ESLint entry point");

function record(input, boundary, packet) {
  const directory = process.env.VIZE_LINT_PROCESSOR_CAPTURE_DIR;
  if (!directory) return;
  fs.mkdirSync(directory, { recursive: true });
  const body = fs.readFileSync(path.join(input.cwd, input.filename), "utf8");
  fs.writeFileSync(
    path.join(directory, `${input.filename}-${boundary}.json`),
    `${JSON.stringify(
      {
        producerSha256: digest(producer),
        embeddedSha256: digest(embedded),
        providers,
        wholeInput: { cwd: input.cwd, filename: input.filename, body, sha256: digest(body) },
        packet,
      },
      null,
      2,
    )}\n`,
  );
}

function fixture(filename, body) {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-processor-"));
  fs.writeFileSync(path.join(cwd, filename), body);
  return { cwd, filename, clean: () => fs.rmSync(cwd, { recursive: true, force: true }) };
}

function strictUnusable(classified, expectedReason) {
  const artifact = {
    project: "authored-raw-boundary",
    files: { comparedCount: 1, expectedCount: 1 },
    baseline: { comparedRuleCount: Object.keys(rules).length },
    divergence: {
      summary: {
        falsePositiveCount: 0,
        falseNegativeCount: 0,
        baselineParseErrorCount: classified.parseErrorCount,
        baselineInvalidRangeCount: classified.invalidRangeCount,
      },
    },
  };
  const budget = evaluateBudget(artifact);
  assert.equal(budget.verdict, "unusable");
  assert.equal(budget.passed, false);
  assert.equal(budget.unusableReason, expectedReason);
  assert.equal(budget.maxFalsePositiveCount, 0);
  assert.equal(budget.maxFalseNegativeCount, 0);
  assert.throws(() => assertBudgetsPassed([{ ...artifact, budget }]), /baseline is unusable/u);
  return budget;
}

function actual(input, selected) {
  const child = spawnSync(process.execPath, ["--input-type=module", "--eval", embedded], {
    cwd: root,
    input: JSON.stringify({
      benchPackageJson: path.join(root, "tools/benchmarks/scripts/package.json"),
      cwd: input.cwd,
      files: [input.filename],
      rules: selected,
    }),
    encoding: "utf8",
    timeout: 30000,
  });
  record(input, "typescript-process", {
    selected,
    status: child.status,
    signal: child.signal,
    error: child.error?.message,
    stdout: child.stdout,
    stderr: child.stderr,
  });
  assert.equal(child.error, undefined, child.error?.message);
  assert.equal(child.status, 0, `${child.stdout}\n${child.stderr}`);
  assert.equal(child.stderr, "");
  const packet = JSON.parse(child.stdout);
  assert.equal(packet.version, runtime.version);
  return { results: packet.results, droppedConfigMessageCount: packet.droppedConfigMessageCount };
}

async function reference(input, selected) {
  const config = baselineConfig(runtime, selected);
  // The independent public processor, with exactly the caller's selected rules.
  config[0].processor = base.processor;
  const eslint = new runtime.ESLint({
    cwd: input.cwd,
    overrideConfigFile: true,
    overrideConfig: config,
    errorOnUnmatchedPattern: false,
  });
  return retainEnabledFindings(await eslint.lintFiles([input.filename]), selected);
}

async function paired(input, selected = rules) {
  // Match the producer's actual JSON transport, including the public getter
  // values; JSON cannot carry an undefined deprecated-rule metadata field.
  const transported = (packet) => JSON.parse(JSON.stringify(packet));
  const expected = transported(await reference(input, selected));
  const measured = [
    actual(input, selected),
    transported(await runBaseline(runtime, input.cwd, [input.filename], selected)),
  ];
  record(input, "whole-results-before-collection", {
    selected,
    publicProcessor: expected,
    actualTypeScript: measured[0],
    compatibility: measured[1],
  });
  for (const packet of measured) {
    assert.deepEqual(packet, expected, "whole real provider result and source custody");
  }
  return measured;
}

const controls = {
  unsuppressed: '<template>\n<div v-html="html" />\n</template>\n',
  nextLine:
    '<template>\n<!-- eslint-disable-next-line vue/no-v-html -->\n<div v-html="html" />\n<p v-html="other" />\n</template>\n',
  sameLine:
    '<template>\n<div v-html="html" /> <!-- eslint-disable-line vue/no-v-html -->\n<p v-html="other" />\n</template>\n',
  reenable:
    '<template>\n<!-- eslint-disable vue/no-v-html -->\n<div v-html="html" />\n<!-- eslint-enable vue/no-v-html -->\n<p v-html="other" />\n</template>\n',
  allRules:
    '<template>\n<!-- eslint-disable -->\n<div v-html="html" />\n<!-- eslint-enable -->\n<p v-html="other" />\n</template>\n',
  scopedRule:
    '<template>\n<!-- eslint-disable vue/no-v-html -->\n<div v-html="html">child</div>\n<!-- eslint-enable vue/no-v-html -->\n<p v-html="other" />\n</template>\n',
};

for (const [name, body] of Object.entries(controls)) {
  await test(`both real baseline producers honor complete ${name} directive output`, async () => {
    const input = fixture(`${name}.vue`, body);
    try {
      for (const packet of await paired(input)) {
        const messages = packet.results[0].messages;
        assert.equal(
          messages.some((message) => message.ruleId === "vue/comment-directive"),
          false,
        );
        const finding = messages.filter((message) => message.ruleId === "vue/no-v-html");
        assert.equal(finding.length, 1, "the independent unsuppressed control remains visible");
        const expectedLine =
          name === "nextLine" ? 4 : name === "sameLine" ? 3 : name === "unsuppressed" ? 2 : 5;
        assert.equal(finding[0].line, expectedLine);
        assert.equal(finding[0].severity, 1, "retain the selected warning severity");
        if (name === "scopedRule") {
          assert.equal(
            messages.filter((message) => message.ruleId === "vue/no-child-content").length,
            1,
            "disabling one rule must not hide another real finding in that region",
          );
        }
      }
    } finally {
      input.clean();
    }
  });
}

await test("the complete original pinned ThemeIcon no longer leaks processor controls", async () => {
  const original = JSON.parse(
    fs.readFileSync(
      path.join(root, "tests/tooling/fixtures/lint-baseline-processor/ThemeIcon.json"),
      "utf8",
    ),
  );
  assert.equal(original.repository, "vueComponent/ant-design-vue");
  assert.equal(original.revision, "7483836f0adac76516e527df893c3d84a7cd4005");
  assert.equal(original.path, "site/src/layouts/icons/ThemeIcon.vue");
  const body = Buffer.from(original.source, "utf8");
  assert.equal(body.length, 1350);
  assert.equal(original.sourceSha256, digest(body));
  assert.equal(
    createHash("sha256").update(body).digest("hex"),
    "eb0f821769f5aa61f9819562c9420c411cf755240f2f33564b7483611f38423b",
    "whole ant-design-vue source at7483836f0adac76516e527df893c3d84a7cd4005",
  );
  const input = fixture("ThemeIcon.vue", body);
  try {
    for (const packet of await paired(input)) {
      assert.equal(
        packet.results[0].messages.some((message) => message.ruleId === "vue/comment-directive"),
        false,
      );
    }
  } finally {
    input.clean();
  }
});

await test("the processor does not enable new default rules or alter the default vector", async () => {
  assert.equal(Object.hasOwn(defaults, "vue/comment-directive"), false);
  assert.equal(Object.hasOwn(defaults, "vue/jsx-uses-vars"), false);
  assert.equal(baselineConfig(runtime, defaults)[0].rules, defaults);
  const input = fixture("default.vue", controls.nextLine);
  try {
    for (const packet of await paired(input, defaults)) {
      assert.equal(
        packet.results[0].messages.filter((message) => message.ruleId === "vue/no-v-html").length,
        2,
        "keep the original caller-selected default rules; do not spread public base rules",
      );
    }
  } finally {
    input.clean();
  }
});

await test("actual malformed source retains its whole fatal packet before strict classification", async () => {
  const input = fixture(
    "malformed.vue",
    '<template><div /></template>\n<script setup lang="ts">const broken = ;</script>\n',
  );
  try {
    for (const packet of await paired(input)) {
      const raw = structuredClone(packet.results);
      assert.equal(raw[0].fatalErrorCount, 1);
      assert.equal(raw[0].messages.length, 1);
      assert.equal(raw[0].messages[0].ruleId, null);
      assert.equal(raw[0].messages[0].fatal, true);
      const classified = collectBaselineFindings(packet.results, input.cwd);
      assert.equal(classified.parseErrorCount, 1);
      assert.equal(classified.findings.length, 0);
      const budget = strictUnusable(
        classified,
        "eslint-plugin-vue could not parse 1 compared file(s)",
      );
      record(input, "strict-fatal-boundary", { raw, classified, budget });
      assert.deepEqual(
        packet.results,
        raw,
        "classification must not mutate the complete raw packet",
      );
    }
  } finally {
    input.clean();
  }
});

await test("a malformed range in a retained real finding stays visible and counted", async () => {
  const input = fixture("bad-range.vue", controls.unsuppressed);
  try {
    const packet = actual(input, rules);
    const real = packet.results[0].messages.find((message) => message.ruleId === "vue/no-v-html");
    assert.ok(real);
    // Explicit mutation of a real provider finding; this is not the missing original six.
    const malformed = { ...real, line: 0 };
    assert.deepEqual(runtime.plugin.processors.vue.postprocess([[malformed]]), [malformed]);
    const raw = [{ ...packet.results[0], messages: [malformed] }];
    const retained = retainEnabledFindings(raw, rules);
    assert.deepEqual(retained.results, raw);
    assert.equal(retained.droppedConfigMessageCount, 0);
    const classified = collectBaselineFindings(retained.results, input.cwd);
    assert.equal(classified.invalidRangeCount, 1);
    assert.equal(classified.findings.length, 0);
    const budget = strictUnusable(
      classified,
      "eslint-plugin-vue reported 1 finding(s) with invalid source ranges",
    );
    assert.deepEqual(
      retained.results,
      raw,
      "never silently drop bad raw data at the producer boundary",
    );
    record(input, "explicit-malformed-range-boundary", { raw, retained, classified, budget });
  } finally {
    input.clean();
  }
});
