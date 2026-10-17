import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { parse } from "yaml";

const read = (path: string) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
const action = parse(read(".github/actions/test-native-vue1-text-document/action.yml"));
const combined = parse(read(".github/actions/test-native-historical-text-documents/action.yml"));
const outer = parse(read(".github/actions/test-native-historical-sfc-documents/action.yml"));
const workflow = parse(read(".github/workflows/check.yml"));
const worker = parse(read(".github/actions/test-full-tooling/action.yml"));
const select = action.runs.steps.find((step: { id?: string }) => step.id === "select");

function selection(event: string, paths: string[], gitStatus = 0, selector = select) {
  const directory = mkdtempSync(join(tmpdir(), "vue1-capture-select-"));
  try {
    const result = spawnSync(
      "bash",
      [
        "-e",
        "-o",
        "pipefail",
        "-c",
        `
      git() {
        [[ "$1" == diff && "$2" == --name-only && "$3" == "$BASE_SHA" && "$4" == "$GITHUB_SHA" ]] || return 91
        if [[ "$MOCK_GIT_STATUS" != 0 ]]; then return "$MOCK_GIT_STATUS"; fi
        printf '%s' "$MOCK_CHANGED_PATHS"
      }
      ${selector.run}
    `,
      ],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_EVENT_NAME: event,
          BASE_SHA: "1111111111111111111111111111111111111111",
          GITHUB_SHA: "2222222222222222222222222222222222222222",
          RUNNER_TEMP: directory,
          GITHUB_OUTPUT: join(directory, "output"),
          MOCK_GIT_STATUS: String(gitStatus),
          MOCK_CHANGED_PATHS: `${paths.join("\n")}\n`,
        },
      },
    );
    return {
      status: result.status,
      output: result.status === 0 ? readFileSync(join(directory, "output"), "utf8") : null,
    };
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

test("full tooling captures historical documents once with the original Vue2 order and browser", () => {
  assert.deepEqual(
    combined.runs.steps.map((step: { uses: string }) => step.uses),
    [
      "./.github/actions/test-native-vue2-text-document",
      "./.github/actions/test-native-vue1-text-document",
    ],
  );
  for (const step of combined.runs.steps)
    assert.deepEqual(step.with, { "comparison-base": "${{ inputs.comparison-base }}" });
  assert.deepEqual(
    outer.runs.steps.map((step: { uses: string }) => step.uses),
    [
      "./.github/actions/test-native-historical-text-documents",
      "./.github/actions/test-native-vue2-scriptless-sfc",
    ],
  );
  assert.equal(outer.runs.steps[1].if, "${{ always() }}");
  for (const step of outer.runs.steps)
    assert.deepEqual(step.with, { "comparison-base": "${{ inputs.comparison-base }}" });
  assert.equal(
    workflow.jobs["test-scripts"].if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.equal(
    workflow.jobs["test-scripts"].steps.filter(
      (step: any) => step.uses === "./.github/actions/test-full-tooling",
    ).length,
    1,
  );
  const steps = worker.runs.steps;
  const hooks = steps.filter(
    (step: any) => step.uses === "./.github/actions/test-native-historical-sfc-documents",
  );
  assert.equal(hooks.length, 1);
  assert.equal(hooks[0].if, "${{ always() && inputs.shard == '1' }}");
  assert.deepEqual(hooks[0].with, {
    "comparison-base": "0000000000000000000000000000000000000000",
  });
  const browser = steps.findIndex(
    (step: any) => step.uses === "./.github/actions/install-formatter-css-browser",
  );
  const capture = steps.indexOf(hooks[0]);
  const tests = steps.findIndex((step: any) => /\btest:scripts:planned\b/.test(step.run ?? ""));
  assert.ok(browser >= 0 && browser < capture && capture < tests);
});

test("all historical selectors capture full qualification without a comparison diff", () => {
  for (const name of ["vue1-text-document", "vue2-text-document", "vue2-scriptless-sfc"]) {
    const leaf = parse(read(`.github/actions/test-native-${name}/action.yml`));
    const selector = leaf.runs.steps.find((step: { id?: string }) => step.id === "select");
    for (const event of ["merge_group", "schedule", "workflow_dispatch"])
      assert.deepEqual(
        selection(event, [], 42, selector),
        { status: 0, output: "selected=true\n" },
        `${name}: ${event}`,
      );
  }
});

test("the actual PR source selector fails closed on missing comparison authority", () => {
  for (const path of [
    "crates/vize_glyph/src/native_doc.rs",
    "crates/vize_glyph/src/native_doc/vue1_text.rs",
    "crates/vize_glyph/src/native_doc/expression/origin.rs",
    "crates/vize_glyph/src/native_doc/printer.rs",
    "crates/vize_glyph/tests/native_vue1_text_document/custody.rs",
    "crates/vize_glyph/tests/fixtures/native-vue1-text-doc-1.0.28.json",
    "davinci/vize_l1/src/dialect/vue1/surface/body.rs",
    "davinci/vize_l1/src/embed/syntax/borrowed.rs",
    "davinci/vize_l0/src/source_frame.rs",
    "vendor/oxc_parser/src/lib.rs",
    "tests/tooling/native-vue1-text-document-oracle.test.ts",
    "tests/tooling/native-vue1-text-document-capture-workflow.test.ts",
    "tests/tooling/support/vue1-pinned-getter.ts",
    "tests/tooling/support/vue1-text-document-judge.ts",
    "tests/_fixtures/reference/vue1/vue.common.cjs.gz",
    ".github/actions/test-native-historical-text-documents/action.yml",
    ".github/workflows/pr-source-checks.yml",
  ])
    assert.deepEqual(
      selection("pull_request", [path]),
      { status: 0, output: "selected=true\n" },
      path,
    );
  for (const paths of [[], ["docs/unrelated.md"], ["crates/vize_canon/src/lib.rs"]])
    assert.deepEqual(selection("pull_request", paths), { status: 0, output: "selected=false\n" });
  assert.deepEqual(selection("pull_request", ["crates/vize_glyph/src/native_doc.rs"], 42), {
    status: 42,
    output: null,
  });
});

test("dedicated capture retains both complete packets and both process failures", () => {
  const capture = action.runs.steps.find((step: any) => step.env?.VIZE_GLYPH_VUE1_TEXT_CAPTURE);
  assert.equal(capture.if, "${{ steps.select.outputs.selected == 'true' }}");
  assert.equal(capture.env.VIZE_GLYPH_VUE1_TEXT_REQUIRE_CAPTURE, "1");
  assert.equal(
    capture.env.VIZE_GLYPH_VUE1_TEXT_SOURCE_HEAD,
    "${{ github.event.pull_request.head.sha || github.sha }}",
  );
  assert.ok(
    capture.run.includes(
      "whole_lf_goldens_idempotence_and_capture_use_only_original_callback_documents -- --exact",
    ),
  );
  assert.ok(capture.run.includes("rust_status=$?"));
  assert.ok(capture.run.includes("oracle_status=$?"));
  assert.ok(capture.run.includes('"$rust_status" != 0 || "$oracle_status" != 0'));
  assert.ok(capture.run.includes('test -s "$VIZE_GLYPH_VUE1_TEXT_CAPTURE"'));
  assert.ok(capture.run.includes('test -s "$VIZE_GLYPH_VUE1_TEXT_ORACLE_CAPTURE"'));
  const upload = action.runs.steps.find((step: any) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  assert.equal(upload.if, "${{ always() && steps.select.outputs.selected == 'true' }}");
  assert.equal(upload.with["if-no-files-found"], "error");
  assert.deepEqual(upload.with.path.trim().split("\n"), [
    "${{ runner.temp }}/native-vue1-text-documents.json",
    "${{ runner.temp }}/native-vue1-text-document-getters.json",
  ]);
});
