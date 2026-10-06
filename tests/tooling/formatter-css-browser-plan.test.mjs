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
  const plan = source.indexOf("- name: Plan CSS formatter browser dependency");
  const install = source.indexOf("- name: Install selected CSS formatter reference browser");
  const selected = source.indexOf("- name: Test selected PR tooling scripts");
  const full = source.indexOf("- name: Test tooling scripts");
  assert.ok(plan >= 0 && plan < install && install < selected && selected < full);
  assert.match(
    source.slice(install, selected),
    /steps\.formatter-css-browser\.outputs\.required == 'true'/,
  );
  const check = read(".github/workflows/check.yml");
  assert.ok(
    check.indexOf("- name: Install CSS formatter reference browser") <
      check.indexOf("- name: Test release scripts"),
  );
  const action = read(".github/actions/install-formatter-css-browser/action.yml");
  assert.match(action, /setup-ubuntu-archive/);
  assert.match(
    action,
    /vize-ci-apt-retry vp exec --filter '\.\/tests' -- playwright install --with-deps chromium/,
  );
});
