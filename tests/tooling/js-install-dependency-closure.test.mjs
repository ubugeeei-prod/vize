import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { testedPackageGroups } from "../../tools/config/vite-plus/tested-package-groups.ts";
import { jsInstallFilters } from "../../tools/support/compat/github/install-js-checks.ts";
import {
  compareInstallClosure,
  currentInstallSource,
  historicalInstaller,
  inspectInstallRecipe,
  prepareInstallRecipe,
} from "../../tools/support/compat/github/js-install-closure.mjs";

const compareNames = (left, right) => (left < right ? -1 : left > right ? 1 : 0);

void test("the pinned package manager selects private transitive dependencies for every JS installer group", () => {
  const original = process.cwd();
  const fixture = mkdtempSync(join(tmpdir(), "vize-install-共有 "));
  const manager = JSON.parse(
    readFileSync(new URL("../../package.json", import.meta.url)),
  ).packageManager;
  const special = new Map([
    ["fresco-conformance", ["./npm/fresco-native"]],
    ["ui-conformance", ["./npm/ui"]],
    ["original-project", ["./npm/oxlint"]],
  ]);
  const groups = new Map([
    ...testedPackageGroups.map(({ name, packages, prebuilt }) => [
      name,
      [...packages, ...prebuilt],
    ]),
    ...special,
  ]);
  const paths = new Set([...groups.values()].flat());
  const native = "./npm/native";
  const uiFixture = "./npm/builder/vite/example";
  const names = new Map(
    [...paths, native, uiFixture].map((path, index) => [path, `fixture-${index}`]),
  );
  const write = (path, value) => {
    const file = join(fixture, path, "package.json");
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, JSON.stringify({ version: "1.0.0", ...value }));
  };
  const select = (filters) =>
    JSON.parse(
      execFileSync(
        "pnpm",
        ["list", ...filters.flatMap((filter) => ["--filter", filter]), "--depth", "-1", "--json"],
        { cwd: fixture, encoding: "utf8" },
      ),
    )
      .map(({ name }) => name)
      .sort(compareNames);
  try {
    write(".", { name: "fixture-root", private: true, packageManager: manager });
    writeFileSync(
      join(fixture, "pnpm-workspace.yaml"),
      "packages:\n  - 'npm/*'\n  - 'npm/builder/*'\n  - 'npm/builder/vite/example'\n  - 'npm/framework/*'\n  - 'npm/compose/*'\n  - 'packages/*'\n",
    );
    for (const [path, name] of names) {
      write(path, { name, devDependencies: { "fixture-shared": "workspace:*" } });
    }
    write("packages/shared", {
      name: "fixture-shared",
      private: true,
      dependencies: { "fixture-leaf": "workspace:*" },
      optionalDependencies: { "fixture-optional": "workspace:*" },
    });
    write("packages/共有 leaf", { name: "fixture-leaf", private: true });
    write("packages/optional", { name: "fixture-optional", private: true });
    write("packages/unrelated", { name: "fixture-unrelated", private: true });
    process.chdir(fixture);
    // `vp run` exposes the package-local `vp`, which has no package-manager commands.
    // Invoke pnpm itself and prove its actual version still honors the fixture's pin.
    assert.equal(
      execFileSync("pnpm", ["--version"], { cwd: fixture, encoding: "utf8" }).trim(),
      manager.slice("pnpm@".length),
    );

    // Preserve the original failure as a causal control against the actual pnpm matcher.
    const rspack = "./npm/builder/rspack";
    assert.deepEqual(select([`${rspack}...`]), [names.get(rspack)]);
    assert.deepEqual(
      select([`{${rspack}}...`]),
      [names.get(rspack), "fixture-shared", "fixture-leaf", "fixture-optional"].sort(compareNames),
    );
    for (const [group, packages] of groups) {
      const roots = [...packages, native, ...(group === "ui" ? [uiFixture] : [])];
      assert.deepEqual(
        select(jsInstallFilters(group)),
        [
          ...new Set([
            "fixture-root",
            ...roots.map((path) => names.get(path)),
            "fixture-shared",
            "fixture-leaf",
            "fixture-optional",
          ]),
        ].sort(compareNames),
        group,
      );
    }
    assert.throws(() => jsInstallFilters("unknown"), /unknown JS check group/);
  } finally {
    process.chdir(original);
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("paired install receipts are bound to the current event rather than each other's labels", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-install-authority-"));
  const authority = currentInstallSource();
  const actor = authority.source.revision;
  const trigger = "b".repeat(40);
  const eventFile = join(fixture, "event.json");
  const environment = {
    GITHUB_SHA: actor,
    GITHUB_EVENT_NAME: "pull_request",
    GITHUB_EVENT_PATH: eventFile,
  };
  const packets = ["before", "after"].map((role) => ({
    schema: "vize.js-install-closure.source-recipe.v2",
    role,
    complete: true,
    githubSha: actor,
    observerActor: actor,
    source: authority.source,
    recipe: role === "before" ? historicalInstaller : authority.installer,
    physicalChanges: role === "before" ? [historicalInstaller.path] : [],
    event: "pull_request",
    triggerHead: trigger,
    inputs: authority.inputs,
    projects: role === "before" ? ["consumer"] : ["@vizejs/builder-shared", "consumer"],
    install: { status: 0 },
    issuer:
      role === "before"
        ? { status: 1, code: "ERR_MODULE_NOT_FOUND" }
        : { status: 0, version: "3.5.1" },
    tests: { status: role === "before" ? 1 : 0 },
  }));
  const save = (values) => {
    for (const packet of values) {
      mkdirSync(join(fixture, packet.role), { recursive: true });
      writeFileSync(join(fixture, packet.role, "qualification.json"), JSON.stringify(packet));
    }
  };
  try {
    writeFileSync(eventFile, JSON.stringify({ pull_request: { head: { sha: trigger } } }));
    save(packets);
    compareInstallClosure(fixture, environment);
    save(packets.map((packet) => ({ ...packet, inputs: { lock: "equally forged" } })));
    assert.throws(
      () => compareInstallClosure(fixture, environment),
      /actual current source inputs/,
    );
    save(packets);
    for (const [change, message] of [
      [(packet) => ({ ...packet, schema: "historical-v1" }), /explicit source\/recipe schema/],
      [
        (packet) => ({
          ...packet,
          source: { ...packet.source, revision: historicalInstaller.revision },
        }),
        /same actual current source/,
      ],
      [
        (packet) => ({ ...packet, source: { ...packet.source, tree: "a".repeat(40) } }),
        /same actual current source/,
      ],
      [
        (packet) => ({ ...packet, recipe: { ...packet.recipe, sha256: "a".repeat(64) } }),
        /authenticated installer recipe/,
      ],
      [
        (packet) => ({ ...packet, recipe: { ...packet.recipe, gitBlob: "a".repeat(40) } }),
        /authenticated installer recipe/,
      ],
      [
        (packet) => ({ ...packet, recipe: { ...packet.recipe, revision: "a".repeat(40) } }),
        /authenticated installer recipe/,
      ],
      [
        (packet) => ({ ...packet, physicalChanges: [...packet.physicalChanges, "package.json"] }),
        /only the declared recipe overlay/,
      ],
      [
        (packet) => ({ ...packet, inputs: { ...packet.inputs, foreign: "a".repeat(64) } }),
        /actual current source inputs/,
      ],
    ]) {
      save(packets.map(change));
      assert.throws(() => compareInstallClosure(fixture, environment), message);
    }
    save(packets);
    save(packets.map((packet) => ({ ...packet, triggerHead: "c".repeat(40) })));
    assert.throws(() => compareInstallClosure(fixture, environment), /triggerHead/);
    save(packets.map((packet) => ({ ...packet, event: "workflow_dispatch", triggerHead: null })));
    assert.throws(() => compareInstallClosure(fixture, environment), /event/);
    assert.throws(() => compareInstallClosure(fixture, {}), /current GitHub actor SHA/);
    assert.throws(
      () => compareInstallClosure(fixture, { ...environment, GITHUB_EVENT_NAME: "push" }),
      /allowed GitHub event/,
    );
    writeFileSync(eventFile, JSON.stringify({ pull_request: { head: { sha: "bad" } } }));
    assert.throws(() => compareInstallClosure(fixture, environment), /current PR trigger SHA/);
    writeFileSync(eventFile, "{}");
    assert.throws(() => compareInstallClosure(fixture, environment), /current PR trigger SHA/);
    const manual = { ...environment, GITHUB_EVENT_NAME: "workflow_dispatch" };
    compareInstallClosure(fixture, manual);
    save(packets.map((packet) => ({ ...packet, event: "workflow_dispatch" })));
    assert.throws(() => compareInstallClosure(fixture, manual), /triggerHead/);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("a current source permits only the exact historical installer overlay", () => {
  const source = mkdtempSync(join(tmpdir(), "vize-current-recipe-"));
  const git = (...args) => execFileSync("git", args, { cwd: source, encoding: "utf8" }).trim();
  const files = [
    "package.json",
    "pnpm-workspace.yaml",
    "pnpm-lock.yaml",
    "npm/builder/shared/package.json",
    "npm/builder/shared/mapped-module.ts",
    "npm/builder/rspack/package.json",
    "npm/builder/rspack/src/shared/source-map.test.ts",
    "tools/config/vite-plus/tested-package-groups.ts",
    historicalInstaller.path,
  ];
  try {
    git("init", "--quiet");
    git("config", "user.name", "Fixture");
    git("config", "user.email", "fixture@example.invalid");
    for (const file of files) {
      mkdirSync(dirname(join(source, file)), { recursive: true });
      writeFileSync(join(source, file), `current source ${file}\n`);
    }
    git("add", ".");
    git("commit", "--quiet", "-m", "test: current source fixture");
    const current = currentInstallSource(source);
    assert.deepEqual(
      prepareInstallRecipe("after", source, current.source.revision).recipe,
      current.installer,
    );
    assert.throws(
      () => prepareInstallRecipe("before", source, historicalInstaller.revision),
      /same actual current source/,
    );
    const before = prepareInstallRecipe("before", source, current.source.revision);
    assert.deepEqual(before.authority, current);
    assert.deepEqual(before.recipe, historicalInstaller);
    assert.deepEqual(before.changes, [historicalInstaller.path]);
    assert.throws(
      () => inspectInstallRecipe("after", source, current),
      /only the declared recipe overlay/,
    );
    const original = readFileSync(join(source, historicalInstaller.path));
    writeFileSync(
      join(source, historicalInstaller.path),
      Buffer.concat([original, Buffer.from("\nchanged")]),
    );
    assert.throws(
      () => inspectInstallRecipe("before", source, current),
      /exact physical installer recipe/,
    );
    writeFileSync(join(source, historicalInstaller.path), original);
    writeFileSync(join(source, "package.json"), "changed current input");
    assert.throws(
      () => inspectInstallRecipe("before", source, current),
      /actual current source inputs/,
    );
    git("restore", "package.json");
    writeFileSync(
      join(source, "tools/config/vite-plus/tested-package-groups.ts"),
      "changed import closure",
    );
    assert.throws(
      () => inspectInstallRecipe("before", source, current),
      /only the declared recipe overlay/,
    );
    git("restore", "tools/config/vite-plus/tested-package-groups.ts");
    writeFileSync(join(source, "unexpected.txt"), "unknown source");
    assert.throws(() => inspectInstallRecipe("before", source, current), /untracked source change/);
    rmSync(join(source, "unexpected.txt"));
    assert.throws(
      () => inspectInstallRecipe("unknown", source, current),
      /declared installer role/,
    );
    git("restore", historicalInstaller.path);
    assert.deepEqual(inspectInstallRecipe("after", source, current).recipe, current.installer);
  } finally {
    rmSync(source, { recursive: true, force: true });
  }
});
