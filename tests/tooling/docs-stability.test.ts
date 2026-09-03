import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const rustStabilityLink =
  "https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers";

type CargoPackage = {
  manifest_path: string;
  metadata: { vize?: { stability?: string } };
  name: string;
  publish: string[] | null;
  readme: string | null;
  targets: Array<{ crate_types: string[]; kind: string[]; src_path: string }>;
};

type RustCrateRow = {
  audience: string;
  crateName: string;
  deprecation: string;
  entrypoint: string;
  tier: string;
};

test("stability page documents v1 alpha support tiers", () => {
  const stability = fs.readFileSync(path.join(root, "docs/content/stability.md"), "utf8");

  assert.match(stability, /# Stability/);
  assert.match(stability, /v1 alpha/);
  assert.match(stability, /Node 22/);
  assert.match(stability, /`oxlint-plugin-vize`[\s\S]*`\^22 \|\| >= 24`/);
  assert.match(stability, /linux-arm64-gnu/);
  assert.match(stability, /win32-arm64-msvc/);
  assert.match(stability, /linux-x64-musl/);
  assert.match(stability, /linux-arm64-musl/);
  assert.match(stability, /`vize --version`/);
  assert.match(stability, /`vize check`/);
  assert.match(stability, /`@vizejs\/native` through both `require` and `import`/);

  for (const tier of ["Alpha-supported", "Compatibility preview", "Experimental", "Incubating"]) {
    assert.match(stability, new RegExp(`\\| ${tier}\\s+\\|`));
  }

  for (const packageName of [
    "vize",
    "@vizejs/native",
    "@vizejs/vite-plugin",
    "@vizejs/unplugin",
    "@vizejs/rspack-plugin",
    "@vizejs/nuxt",
    "@vizejs/nuxt-lint-config",
    "@vizejs/vite-plugin-musea",
    "@vizejs/wasm",
    "@vizejs/fresco",
  ]) {
    assert.match(stability, new RegExp(escapeRegExp(`\`${packageName}\``)));
  }
});

test("Rust crate stability table matches Cargo metadata and crate documentation", () => {
  const stability = fs.readFileSync(path.join(root, "docs/content/stability.md"), "utf8");
  const table = stability.match(
    /<!-- rust-crate-support:start -->(?<body>[\s\S]*?)<!-- rust-crate-support:end -->/,
  )?.groups?.body;
  assert.ok(table, "missing checked Rust crate support table");

  const rows = table
    .split("\n")
    .filter((line) => /^\| `vize_[^`]+`/.test(line))
    .map(parseRustCrateRow);
  const rowsByCrate = new Map(rows.map((row) => [row.crateName, row]));
  assert.equal(rowsByCrate.size, rows.length, "each publishable crate must have exactly one row");

  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: root,
      encoding: "utf8",
    }),
  ) as { packages: CargoPackage[] };
  const publishableCrates = metadata.packages.filter(
    (pkg) =>
      path.relative(root, pkg.manifest_path).startsWith(`crates${path.sep}`) &&
      (pkg.publish === null || pkg.publish.length > 0),
  );

  assert.deepEqual(
    [...rowsByCrate.keys()].toSorted(),
    publishableCrates.map((pkg) => pkg.name).toSorted(),
  );

  const tierLabels = new Map([
    ["alpha-supported", "Alpha-supported"],
    ["compatibility-preview", "Compatibility preview"],
    ["experimental", "Experimental"],
    ["incubating", "Incubating"],
  ]);

  for (const pkg of publishableCrates) {
    const row = rowsByCrate.get(pkg.name);
    assert.ok(row, `missing support row for ${pkg.name}`);
    const tier = pkg.metadata.vize?.stability;
    assert.ok(tier, `${pkg.name} must declare package.metadata.vize.stability`);
    assert.equal(row.tier, tierLabels.get(tier), `${pkg.name} tier drift`);
    assert.ok(row.audience, `${pkg.name} must name its intended audience`);
    assert.match(row.entrypoint, /^`vize_[^`]+/);
    assert.ok(row.deprecation, `${pkg.name} must define a deprecation policy`);

    assert.ok(pkg.readme, `${pkg.name} must ship a README`);
    const readmePath = path.resolve(path.dirname(pkg.manifest_path), pkg.readme);
    assert.match(fs.readFileSync(readmePath, "utf8"), new RegExp(escapeRegExp(rustStabilityLink)));

    if (tier === "experimental" || tier === "incubating") {
      const documentedTarget = pkg.targets.find(
        (target) =>
          target.kind.includes("lib") ||
          target.kind.includes("proc-macro") ||
          target.crate_types.includes("rlib") ||
          target.crate_types.includes("proc-macro"),
      );
      assert.ok(documentedTarget, `${pkg.name} must expose a documented library target`);
      assert.match(
        fs.readFileSync(documentedTarget.src_path, "utf8"),
        new RegExp(`\\*\\*${tier === "experimental" ? "Experimental" : "Incubating"}`),
        `${pkg.name} rustdoc must disclose its ${tier} status`,
      );
    }
  }
});

function parseRustCrateRow(line: string): RustCrateRow {
  const cells = line
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
  assert.equal(cells.length, 5, `invalid Rust support table row: ${line}`);
  const crateName = cells[0].match(/^`(?<name>vize_[^`]+)`$/)?.groups?.name;
  assert.ok(crateName, `invalid crate name in support table: ${cells[0]}`);
  return {
    crateName,
    tier: cells[1],
    audience: cells[2],
    entrypoint: cells[3],
    deprecation: cells[4],
  };
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
