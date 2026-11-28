import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { renderRepositoryYaml, type ConfigValue } from "../../tools/support/ci/repository-yaml.ts";

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

test("ordered configuration preserves the distinction between a non-finite float and null", () => {
  const config = { option: null };
  const rendered = renderRepositoryYaml(config, "option: .nan\n");
  assert.deepEqual(parse(rendered), config);
  assert.equal(renderRepositoryYaml(config, rendered), rendered);
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

test("inserting and reordering workflow steps keeps comments on their owning steps", () => {
  const source = [
    "steps:",
    "  - name: Build",
    "    run: build",
    "  - id: publish",
    "    name: Upload",
    "    run: upload # publishes the build",
    "",
  ].join("\n");
  const build = { name: "Build", run: "build" };
  const publish = { id: "publish", name: "Publish", run: "upload --checked" };
  for (const steps of [
    [{ name: "Install", run: "install" }, build, publish],
    [publish, build],
  ]) {
    const rendered = renderRepositoryYaml({ steps }, source, false);
    assert.deepEqual(parse(rendered), { steps });
    assert.match(rendered, /run: upload --checked # publishes the build/u);
    assert.equal(renderRepositoryYaml({ steps }, rendered, false), rendered);
  }
});

test("a new key, a longer flow list and replaced block items keep the rest of the layout", () => {
  const source = [
    "jobs:",
    "  build:",
    "    needs: [version, tag]",
    "    steps:",
    "      - uses: setup # v1",
    "        with: { node-version-file: package.json }",
    "      - uses: scan # v2",
    "        with: { format: spdx }",
    "      - uses: scan # v2",
    "        with: { format: cyclonedx }",
    "      - run: >",
    "          sign",
    "          files",
    "  release:",
    "    runs-on: ubuntu",
    "",
  ].join("\n");
  const config: ConfigValue = {
    jobs: {
      build: {
        needs: ["version", "tag", "scan"],
        steps: [
          { uses: "setup", with: { "node-version-file": "package.json" } },
          { uses: "download", with: { name: "scan" } },
          { run: "sign files\n" },
        ],
      },
      scan: { needs: ["version"], steps: [{ uses: "scan" }] },
      release: { "runs-on": "ubuntu" },
    },
  };
  const rendered = renderRepositoryYaml(config, source);
  assert.deepEqual(parse(rendered), config);
  assert.equal(
    rendered,
    [
      "jobs:",
      "  build:",
      "    needs: [version, tag, scan]",
      "    steps:",
      "      - uses: setup # v1",
      "        with: { node-version-file: package.json }",
      "      - uses: download",
      "        with:",
      "          name: scan",
      "      - run: >",
      "          sign",
      "          files",
      "  scan:",
      "    needs:",
      "      - version",
      "    steps:",
      "      - uses: scan",
      "  release:",
      "    runs-on: ubuntu",
      "",
    ].join("\n"),
  );
  assert.equal(renderRepositoryYaml(config, rendered), rendered);
});

test("a mapping that gains one scalar key gets it in place, in flow and block style", () => {
  const source = [
    "steps:",
    "  - uses: setup # v1",
    "    with: { node-version-file: package.json }",
    "  - uses: setup # v1",
    "    with:",
    "      cache: true",
    "",
  ].join("\n");
  const config: ConfigValue = {
    steps: [
      { uses: "setup", with: { version: "1.8.0", "node-version-file": "package.json" } },
      { uses: "setup", with: { version: "1.8.0", cache: true } },
    ],
  };
  const rendered = renderRepositoryYaml(config, source);
  assert.equal(
    rendered,
    [
      "steps:",
      "  - uses: setup # v1",
      "    with: { version: 1.8.0, node-version-file: package.json }",
      "  - uses: setup # v1",
      "    with:",
      "      version: 1.8.0",
      "      cache: true",
      "",
    ].join("\n"),
  );
  assert.equal(renderRepositoryYaml(config, rendered), rendered);
});
