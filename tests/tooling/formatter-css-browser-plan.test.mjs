import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { needsFormatterCssBrowser } from "../../tools/support/compat/github/formatter-css-browser-plan.mjs";

const browser = "tests/tooling/formatter-css-rule-layout.test.mjs";
const files = ["a.test.mjs", "b.test.mjs", "c.test.mjs", browser];

await test("only the selected browser-owning runner needs Chromium, with complete merge validation", () => {
  for (const tier of ["pr", "merge"]) {
    const plan = { version: 1, tier, tests: files };
    for (let index = 1; index <= 4; index++)
      assert.equal(needsFormatterCssBrowser(plan, files, `${index}/4`, tier), index === 4);
    assert.equal(needsFormatterCssBrowser(plan, files, "", tier), true);
  }
  assert.equal(
    needsFormatterCssBrowser({ version: 1, tier: "pr", tests: [files[0]] }, files, "1/1", "pr"),
    false,
  );
  assert.throws(
    () =>
      needsFormatterCssBrowser(
        { version: 1, tier: "merge", tests: [browser] },
        files,
        "1/1",
        "merge",
      ),
    /retain every test/,
  );
  assert.throws(
    () => needsFormatterCssBrowser({ version: 1, tier: "pr", tests: files }, files, "4/4", "merge"),
    /execution tier/,
  );
});

await test("selected and unsharded workflows provision the actual tests-package browser before execution", () => {
  const read = (file) => fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
  const source = read(".github/workflows/pr-source-checks.yml");
  const prepare = source.indexOf(
    "- name: Prepare checksum-pinned Pkl and upstream lint dependencies",
  );
  const install = source.indexOf("uses: ./.github/actions/install-formatter-css-browser");
  const selected = source.indexOf("- name: Test selected PR tooling scripts");
  const full = source.indexOf("- name: Test tooling scripts");
  assert.ok(prepare >= 0 && prepare < install && install < selected && selected < full);
  assert.match(
    source.slice(prepare, install),
    /VIZE_TOOLING_TEST_PLAN: \$\{\{ runner\.temp \}\}\/tooling-plan\.json/,
  );
  const check = read(".github/workflows/check.yml");
  assert.ok(
    check.indexOf("- name: Install CSS formatter reference browser") >= 0 &&
      check.indexOf("- name: Install CSS formatter reference browser") <
        check.indexOf("- name: Test release scripts"),
  );
  const action = read(".github/actions/install-formatter-css-browser/action.yml");
  assert.ok(action.indexOf("prepare-pkl-schema.mjs") < action.indexOf("id: formatter-css-browser"));
  assert.ok(action.indexOf("id: formatter-css-browser") < action.indexOf("setup-ubuntu-archive"));
  assert.match(
    action,
    /if: env\.VIZE_TOOLING_TEST_PLAN == '' \|\| steps\.formatter-css-browser\.outputs\.required == 'true'/,
  );
  assert.match(action, /setup-ubuntu-archive/);
  assert.match(
    action,
    /vize-ci-apt-retry vp exec --filter '\.\/tests' -- playwright install --with-deps chromium/,
  );
});
