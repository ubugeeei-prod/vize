import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { testedPackageGroups } from "../../tools/config/vite-plus/tested-package-groups.ts";
import { jsInstallFilters } from "../../tools/support/compat/github/install-js-checks.ts";

test("the pinned package manager selects private transitive dependencies for every JS installer group", () => {
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
        "vp",
        [
          "list",
          "--",
          ...filters.flatMap((filter) => ["--filter", filter]),
          "--depth",
          "-1",
          "--json",
        ],
        { cwd: fixture, encoding: "utf8" },
      ),
    )
      .map(({ name }) => name)
      .sort();
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

    // Preserve the original failure as a causal control against the actual pnpm matcher.
    const rspack = "./npm/builder/rspack";
    assert.deepEqual(select([`${rspack}...`]), [names.get(rspack)]);
    assert.deepEqual(
      select([`{${rspack}}...`]),
      [names.get(rspack), "fixture-shared", "fixture-leaf", "fixture-optional"].sort(),
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
        ].sort(),
        group,
      );
    }
    assert.throws(() => jsInstallFilters("unknown"), /unknown JS check group/);
  } finally {
    process.chdir(original);
    rmSync(fixture, { recursive: true, force: true });
  }
});
