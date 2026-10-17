import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { planCheckInputs } from "../../tools/support/compat/github/plan-check-inputs.ts";

await test("prose and tooling metadata do not rebuild product apps", () => {
  assert.deepEqual(planCheckInputs(["docs/content/guide.md", "docs/davinci/decisions/test.md"]), {
    full: false,
    rust: false,
    dependencies: false,
    engines: false,
    apps: false,
  });
  assert.equal(planCheckInputs(["tests/tooling/plan-source-checks.test.ts"]).apps, false);
});

await test("product, compiled documentation and cache changes retain app checks", () => {
  for (const path of [
    "davinci/vize_l2/src/lib.rs",
    "crates/vize/src/main.rs",
    "vendor/lightningcss/src/lib.rs",
    "npm/builder/vite/src/index.ts",
    "npm/cli/schemas/vize.config.schema.json",
    "playground/src/App.vue",
    "npm/compose/core/src/virtual-list.ts",
    "npm/ui/scripts/reference-docs/composable-examples.ts",
    "docs/theme/components/Guide.vue",
    "examples/html-conformance/src/index.html",
    "docs/davinci/plan/canonical-corpus/budgets.toml",
    "docs/davinci/plan/fact-alpha-schemas.md",
    ".github/actions/setup-rust-sticky-cache/action.yml",
  ]) {
    assert.equal(planCheckInputs([path]).apps, true, path);
  }
});

await test("dependency and security changes always audit and check engine compatibility", () => {
  for (const path of [
    "Cargo.lock",
    "davinci/vize_l1/Cargo.toml",
    "npm/cli/package.json",
    "pnpm-lock.yaml",
    "patches/listhen.patch",
    "tools/support/security/node-forge-proof.ts",
    "tools/commands/ci/npm-audit-retry.ts",
  ]) {
    const plan = planCheckInputs([path]);
    assert.equal(plan.dependencies, true, path);
    assert.equal(plan.engines, true, path);
    assert.equal(plan.apps, true, path);
  }
});

await test("npm Vue examples select the current-source opinionated lint consumer", () => {
  for (const path of [
    "npm/compose/core/examples/virtual-list.vue",
    "npm/compose/future/examples/nested/NewExample.vue",
    "npm/ui/core/src/NewComponent.vue",
  ]) {
    assert.deepEqual(
      planCheckInputs([path]),
      {
        full: false,
        rust: false,
        dependencies: false,
        engines: false,
        apps: true,
      },
      path,
    );
  }
});

await test("npm prose and unknown non-SFC extensions do not select extra native app builds", () => {
  for (const path of [
    "npm/compose/core/examples/README.md",
    "npm/compose/core/src/virtual-list.json",
    "npm/ui/scripts/reference-docs/composable-examples.unknown",
  ]) {
    assert.equal(planCheckInputs([path]).apps, false, path);
  }
});

await test("the actual scanner's complete authored input set selects its app consumer", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-authored-source-scope-"));
  const scanner = fileURLToPath(
    new URL("../../tools/commands/ci/opinionated-source-lint.mjs", import.meta.url),
  );
  const roots = [
    "npm/compose/core/src",
    "docs/theme/components",
    "playground/src",
    "examples/vite-musea/src",
    "examples/html-conformance/src",
  ];
  const extensions = [
    "vue",
    "html",
    "htm",
    "js",
    "jsx",
    "mjs",
    "mjsx",
    "cjs",
    "cjsx",
    "ts",
    "tsx",
    "mts",
    "mtsx",
    "cts",
    "ctsx",
  ];
  const sources = roots.flatMap((root) => extensions.map((ext) => `${root}/Authored.${ext}`));
  const excluded = roots.flatMap((root) => [
    `${root}/README.md`,
    `${root}/data.json`,
    `${root}/Unknown.svelte`,
    `${root}/Upper.VUE`,
    `${root}/intentional.fixture.tsx`,
    `${root}/fixtures/invalid.vue`,
    `${root}/__fixtures__/invalid.ts`,
    `${root}/__snapshots__/invalid.html`,
  ]);
  try {
    execFileSync("git", ["init", "-q"], { cwd });
    for (const path of [...sources, ...excluded, "unowned/source.tsx"]) {
      mkdirSync(join(cwd, path, ".."), { recursive: true });
      writeFileSync(join(cwd, path), "controlled tracked source\n");
    }
    execFileSync("git", ["add", "."], { cwd });
    const output = execFileSync(process.execPath, [scanner, "--list-files"], {
      cwd,
      encoding: "utf8",
    });
    assert.deepEqual(output.trim().split("\n"), [...sources].sort());
    for (const path of sources) assert.equal(planCheckInputs([path]).apps, true, path);
    for (const path of ["docs/theme/components/data.json", "unowned/source.tsx"])
      assert.equal(planCheckInputs([path]).apps, false, path);
    // Preserve historical conservative Vue routing independently of fixture exclusions.
    assert.equal(planCheckInputs(["npm/compose/core/fixtures/invalid.vue"]).apps, true);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

await test("actual PR and queue diffs select authored sources after additions and removals", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-authored-source-diff-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const planner = fileURLToPath(
    new URL("../../tools/support/compat/github/plan-check-inputs.ts", import.meta.url),
  );
  const source = "docs/theme/components/Added.tsx";
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    git("remote", "add", "origin", cwd);
    writeFileSync(join(cwd, "README.md"), "initial\n");
    git("add", ".");
    git("commit", "-qm", "initial source");
    let base = git("rev-parse", "HEAD");
    mkdirSync(join(cwd, source, ".."), { recursive: true });
    writeFileSync(join(cwd, source), "export const added = true;\n");
    for (const operation of ["add", "remove"]) {
      if (operation === "remove") rmSync(join(cwd, source));
      git("add", ".");
      git("commit", "-qm", operation);
      const head = git("rev-parse", "HEAD");
      for (const event of ["pull_request", "merge_group"]) {
        const output = join(cwd, `output-${operation}-${event}`);
        const run = spawnSync(process.execPath, [planner], {
          cwd,
          encoding: "utf8",
          env: {
            ...process.env,
            GITHUB_EVENT_NAME: event,
            EVENT_BASE_SHA: base,
            GITHUB_SHA: head,
            PR_HEAD_SHA: head,
            GITHUB_OUTPUT: output,
            GITHUB_STEP_SUMMARY: "",
          },
        });
        assert.equal(run.status, 0, run.stderr);
        assert.equal(
          readFileSync(output, "utf8"),
          "full=false\nrust=false\ndependencies=false\nengines=false\napps=true\n",
          `${operation}: ${event}`,
        );
        rmSync(output);
      }
      base = head;
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

await test("changing shared ownership changes the actual scanner and selector together", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-authored-ownership-mutation-"));
  const helper = "tools/support/github/opinionated-source-inputs.ts";
  const scanner = "tools/commands/ci/opinionated-source-lint.mjs";
  const planner = "tools/support/compat/github/plan-check-inputs.ts";
  try {
    for (const path of [
      helper,
      scanner,
      planner,
      "tools/support/compat/github/comparison-base.ts",
    ]) {
      mkdirSync(join(cwd, path, ".."), { recursive: true });
      writeFileSync(join(cwd, path), readFileSync(new URL(`../../${path}`, import.meta.url)));
    }
    const original = readFileSync(join(cwd, helper), "utf8");
    assert.equal(original.split('  "npm/",').length, 2);
    assert.equal(original.split("vue|html?").length, 3);
    // Mutate only the shared owner, leaving both actual consumer modules intact.
    writeFileSync(
      join(cwd, helper),
      original
        .replace('  "npm/",', '  "npm/",\n  "future-owned/",')
        .replaceAll("vue|html?", "widget|vue|html?"),
    );
    const source = "future-owned/Added.widget";
    for (const path of [source, "future-owned/ignored.fixture.widget", "unowned/Other.widget"]) {
      mkdirSync(join(cwd, path, ".."), { recursive: true });
      writeFileSync(join(cwd, path), "controlled ownership mutation\n");
    }
    execFileSync("git", ["init", "-q"], { cwd });
    execFileSync("git", ["add", "."], { cwd });
    assert.equal(
      execFileSync(process.execPath, [join(cwd, scanner), "--list-files"], {
        cwd,
        encoding: "utf8",
      }),
      `${source}\n`,
    );
    for (const [path, expected] of [
      [source, true],
      ["future-owned/ignored.fixture.widget", false],
      ["unowned/Other.widget", false],
    ] as const) {
      const output = execFileSync(
        process.execPath,
        [
          "--input-type=module",
          "-e",
          "const { planCheckInputs } = await import(process.argv[1]); console.log(JSON.stringify(planCheckInputs([process.argv[2]])));",
          pathToFileURL(join(cwd, planner)).href,
          path,
        ],
        { cwd, encoding: "utf8" },
      );
      assert.equal(JSON.parse(output).apps, expected, path);
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

await test("the actual PR and merge-group planner selects a newly added npm Vue example", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-check-example-plan-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const script = fileURLToPath(
    new URL("../../tools/support/compat/github/plan-check-inputs.ts", import.meta.url),
  );
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    git("remote", "add", "origin", cwd);
    writeFileSync(join(cwd, "README.md"), "before\n");
    git("add", ".");
    git("commit", "-qm", "initial source");
    const base = git("rev-parse", "HEAD");
    const examples = join(cwd, "npm/compose/core/examples");
    mkdirSync(examples, { recursive: true });
    writeFileSync(join(examples, "virtual-list.vue"), "<template><div /></template>\n");
    git("add", ".");
    git("commit", "-qm", "add ordinary Vue example");
    const head = git("rev-parse", "HEAD");
    for (const event of ["pull_request", "merge_group"]) {
      const output = join(cwd, `output-${event}`);
      const run = spawnSync(process.execPath, [script], {
        cwd,
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_EVENT_NAME: event,
          EVENT_BASE_SHA: base,
          GITHUB_SHA: head,
          PR_HEAD_SHA: head,
          GITHUB_OUTPUT: output,
          GITHUB_STEP_SUMMARY: "",
        },
      });
      assert.equal(run.status, 0, run.stderr);
      assert.equal(
        readFileSync(output, "utf8"),
        "full=false\nrust=false\ndependencies=false\nengines=false\napps=true\n",
        event,
      );
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

await test("scheduled and manual qualification never select away checks", () => {
  assert.equal(Object.values(planCheckInputs([], true)).every(Boolean), true);
});

await test("changing selector and task definitions exercises every affected consumer", () => {
  for (const path of [
    ".github/workflows/check.yml",
    "tools/support/compat/github/plan-check-inputs.ts",
    "tools/support/github/opinionated-source-inputs.ts",
    "tools/config/vite-plus/tasks/check.ts",
  ]) {
    const plan = planCheckInputs([path]);
    assert.equal(plan.apps, true, path);
    assert.equal(plan.dependencies, true, path);
    assert.equal(plan.engines, true, path);
  }
  assert.equal(planCheckInputs([".cargo/audit.toml"]).dependencies, true);
});
