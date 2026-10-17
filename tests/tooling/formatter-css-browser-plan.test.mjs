import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { needsFormatterCssBrowser } from "../../tools/support/compat/github/formatter-css-browser-plan.mjs";
import {
  selectToolingShard,
  toolingShardMatrix,
} from "../../tools/support/compat/github/tooling-test-shards.ts";

const browser = "tests/tooling/formatter-css-rule-layout.test.mjs";
const others = Array.from({ length: 40 }, (_, index) => `tests/tooling/${index}.test.mjs`);
const files = [...others.slice(0, 3), browser];

await test("only the selected browser-owning runner needs Chromium, with complete merge validation", () => {
  for (const tier of ["pr", "merge"]) {
    const tests = [...others, browser];
    const plan = { version: 1, tier, tests };
    const shards = toolingShardMatrix(plan).include;
    assert.ok(shards.length > 1);
    const owners = shards.filter(({ index, total }) => {
      const shard = `${index}/${total}`;
      const needed = needsFormatterCssBrowser(plan, tests, shard, tier);
      assert.equal(needed, selectToolingShard(plan, shard).includes(browser));
      return needed;
    });
    assert.equal(owners.length, 1);
    assert.equal(needsFormatterCssBrowser(plan, tests, "", tier), true);
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

await test("the full workflow and its shared worker provision the actual tests-package browser before execution", () => {
  const read = (file) => fs.readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
  const check = read(".github/workflows/check.yml");
  const tooling = check.slice(
    check.indexOf("\n  test-scripts:"),
    check.indexOf("\n  editor-extensions:"),
  );
  assert.match(tooling, /needs: \[full-source-cli, tooling-plan\]/);
  assert.match(tooling, /uses: \.\/\.github\/actions\/test-full-tooling/);
  const worker = read(".github/actions/test-full-tooling/action.yml");
  assert.ok(
    worker.indexOf("- name: Install CSS formatter reference browser") >= 0 &&
      worker.indexOf("- name: Install CSS formatter reference browser") <
        worker.indexOf("- name: Test release scripts"),
  );
  assert.match(worker, /uses: \.\/\.github\/actions\/install-formatter-css-browser/);
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
