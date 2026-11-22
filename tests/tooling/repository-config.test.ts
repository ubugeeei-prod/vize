import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { renderRepositoryYaml } from "../../tools/support/ci/repository-yaml.ts";

test("updating an action pin changes just the scalar and keeps comments, layout, and anchors", () => {
  const source = [
    "# workflow",
    "steps:",
    "  - &checkout { uses: 'old', with: { ref: main } } # stable",
    "  - *checkout",
    "",
  ].join("\n");
  const step = { uses: "new", with: { ref: "main" } };
  const config = { steps: [step, step] };
  assert.equal(renderRepositoryYaml(config, source), source.replace("'old'", "'new'"));
  const independent = { steps: [step, { ...step, uses: "other" }] };
  assert.deepEqual(parse(renderRepositoryYaml(independent, source)), independent);
});

test("ordered configuration preserves key order, scalar types, and multiline commands", () => {
  const config = { a: "false", on: null, run: "first\nsecond\n", b: false };
  const input = "# settings\nb: false\na: true\nrun: |\n  first\n";
  const rendered = renderRepositoryYaml(config, input);
  assert.deepEqual(Object.entries(parse(rendered)), Object.entries(config));
  assert.match(rendered, /# settings\nb: false/u);
  assert.equal(renderRepositoryYaml(config, rendered), rendered);
  assert.deepEqual(parse(renderRepositoryYaml(config)), config);
});

test("Actions mappings may keep their layout, but step order must match the source", () => {
  const input = "name: job\nsteps: [first, second]\n";
  assert.equal(
    renderRepositoryYaml({ steps: ["first", "second"], name: "job" }, input, false),
    input,
  );
  const reordered = { name: "job", steps: ["second", "first"] };
  assert.equal(
    renderRepositoryYaml(reordered, input, false),
    "name: job\nsteps: [second, first]\n",
  );
});

test("a malformed output can be recovered from its Pkl source", () => {
  const config = { steps: ["build"] };
  assert.deepEqual(parse(renderRepositoryYaml(config, "steps: *missing\n")), config);
  assert.deepEqual(parse(renderRepositoryYaml(config, "steps: [\n")), config);
});
