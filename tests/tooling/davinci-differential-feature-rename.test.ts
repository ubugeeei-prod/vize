import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { rewrite, run } from "../../tools/support/levels/rename-differential-features.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));

function read(path: string): string {
  return readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
}

test("differential feature rename is complete in tracked source", () => {
  const output = execFileSync(
    process.execPath,
    ["tools/support/levels/rename-differential-features.ts", "--check"],
    { cwd: root, encoding: "utf8" },
  );
  assert.match(output, /^0 files contain old selectors$/m);
});

const oldName = ["davinci", "differential"].join("-");
const oldDom = ["davinci", "dom", "differential"].join("-");
const sfcPath = "crates/vize_atelier_sfc/Cargo.toml";
const edges = [oldDom, `vize_atelier_ssr/${oldName}`];

test("rewriting preserves both published benchmark edges and rewrites stray selectors", () => {
  const source = read(sfcPath) + `\nprobe = ["${oldDom}", "${oldName}"]\n`;
  const rewritten = rewrite(source, sfcPath);
  const block = rewritten.match(/^davinci-production-bench = \[\n([\s\S]*?)^\]/m)?.[1];
  assert.ok(block);
  for (const edge of edges) assert.equal(block.split(`"${edge}"`).length - 1, 1);
  assert.match(rewritten, /^probe = \["legacy-dom-differential", "legacy-differential"\]$/m);
  assert.equal(rewrite(rewritten, sfcPath), rewritten);
});

test("missing or duplicate benchmark blocks and published members fail before rewriting", () => {
  const source = read(sfcPath);
  const block = source.match(/^davinci-production-bench = \[\n[\s\S]*?^\]/m)?.[0];
  assert.ok(block);
  for (const wrong of [source.replace(block, ""), source + `\n${block}\n`])
    assert.throws(() => rewrite(wrong, sfcPath), /exactly one published SFC benchmark block/);
  for (const edge of edges) {
    const member = `  "${edge}",`;
    assert.throws(
      () => rewrite(source.replace(member, ""), sfcPath),
      /exactly one published benchmark edge/,
    );
    assert.throws(
      () => rewrite(source.replace(member, `${member}\n${member}`), sfcPath),
      /exactly one published benchmark edge/,
    );
  }
});

test("write validates all protected manifests before modifying earlier files", () => {
  const checkout = mkdtempSync(path.join(os.tmpdir(), "vize-published-feature-"));
  try {
    mkdirSync(path.join(checkout, "crates/vize_atelier_sfc"), { recursive: true });
    const earlier = path.join(checkout, "crates/a.rs");
    const original = `// probe ${oldName}\n`;
    writeFileSync(earlier, original);
    writeFileSync(path.join(checkout, sfcPath), "[features]\ndefault = []\n");
    execFileSync("git", ["init", "-q"], { cwd: checkout });
    execFileSync("git", ["add", "crates"], { cwd: checkout });
    assert.throws(() => run("--write", checkout), /exactly one published SFC benchmark block/);
    assert.equal(readFileSync(earlier, "utf8"), original);
  } finally {
    rmSync(checkout, { recursive: true, force: true });
  }
});

test("rename checks the final eligible entry after a tracked inventory larger than one MiB", () => {
  const checkout = mkdtempSync(path.join(os.tmpdir(), "vize-feature-inventory-control-"));
  try {
    execFileSync("git", ["init", "-q"], { cwd: checkout });
    const blob = execFileSync("git", ["hash-object", "-w", "--stdin"], {
      cwd: checkout,
      input: "",
      encoding: "utf8",
    }).trim();
    const padding = Array.from(
      { length: 5_000 },
      (_, index) => `tests/fixtures/${"padding".repeat(29)}-${String(index).padStart(6, "0")}.rs`,
    );
    execFileSync("git", ["update-index", "--add", "-z", "--index-info"], {
      cwd: checkout,
      input: padding.map((file) => `100644 ${blob}\t${file}\0`).join(""),
    });
    const relative = "tests/tooling/zzzz-inventory.ts";
    const final = path.join(checkout, relative);
    const original = `// ${oldName}\n`;
    mkdirSync(path.dirname(final), { recursive: true });
    writeFileSync(final, original);
    execFileSync("git", ["add", relative], { cwd: checkout });
    const args = ["ls-files", "-z", "--", "tests"];
    const inventory = execFileSync("git", args, { cwd: checkout, maxBuffer: 4 * 1024 * 1024 });
    assert.ok(inventory.byteLength > 1024 * 1024);
    assert.equal(inventory.toString("utf8").split("\0").at(-2), relative);
    assert.throws(
      () => execFileSync("git", args, { cwd: checkout }),
      (error: unknown) => (error as NodeJS.ErrnoException).code === "ENOBUFS",
    );
    assert.equal(run("--check", checkout), 1);
    assert.equal(readFileSync(final, "utf8"), original);
    assert.equal(run("--write", checkout), 0);
    assert.equal(readFileSync(final, "utf8"), "// legacy-differential\n");
    assert.equal(run("--check", checkout), 0);
  } finally {
    rmSync(checkout, { recursive: true, force: true });
  }
});

test("Git inventory failure rejects write before changing any file", () => {
  const checkout = mkdtempSync(path.join(os.tmpdir(), "vize-feature-inventory-failure-"));
  try {
    const file = path.join(checkout, "crates/probe.rs");
    const original = `// ${oldName}\n`;
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, original);
    assert.throws(() => run("--write", checkout), /not a git repository/);
    assert.equal(readFileSync(file, "utf8"), original);
  } finally {
    rmSync(checkout, { recursive: true, force: true });
  }
});

test("locked Cargo metadata retains published and current SSR benchmark edges", () => {
  const metadata = JSON.parse(
    execFileSync(
      "cargo",
      ["metadata", "--no-deps", "--locked", "--offline", "--format-version", "1"],
      { cwd: root, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
    ),
  );
  const sfc = metadata.packages.find((pkg: any) => pkg.name === "vize_atelier_sfc");
  const ssr = metadata.packages.find((pkg: any) => pkg.name === "vize_atelier_ssr");
  assert.ok(sfc && ssr);
  assert.deepEqual(sfc.features["davinci-production-bench"], [
    "legacy-dom-differential",
    oldDom,
    "vize_atelier_vapor/davinci-benchmark",
    "vize_atelier_ssr/legacy-differential",
    `vize_atelier_ssr/${oldName}`,
  ]);
  assert.deepEqual(sfc.features[oldDom], ["legacy-dom-differential"]);
  assert.deepEqual(ssr.features[oldName], ["legacy-differential"]);
  assert.deepEqual(ssr.features["legacy-differential"], []);
});

test("renamed features retain the same differential dependency wiring", () => {
  const sfc = read("crates/vize_atelier_sfc/Cargo.toml");
  const core = read("crates/vize_atelier_core/Cargo.toml");
  const vapor = read("crates/vize_atelier_vapor/Cargo.toml");
  const recipe = read(".github/actions/test-rust-workspace-differential/action.yml");

  assert.match(sfc, /^legacy-differential = \[$/m);
  assert.match(sfc, /^legacy-dom-differential = \["vize_atelier_dom\/legacy-differential"\]$/m);
  assert.match(core, /^legacy-differential = \[\]$/m);
  assert.match(vapor, /^legacy-differential = \["vize_atelier_core\/legacy-differential"\]$/m);
  assert.match(recipe, /--features legacy-differential --test davinci_lowering_corpus/u);
  assert.match(recipe, /--features legacy-dom-differential --test davinci_production_reach/u);
  assert.match(
    read("tests/davinci_test_support/src/corpus.rs"),
    /davinci-differential corpus scope/u,
  );
});

test("v0.429.1 Cargo feature names remain temporary aliases", () => {
  const publishedDifferentialAlias = ["davinci", "differential"].join("-");
  const publishedDomAlias = ["davinci", "dom", "differential"].join("-");
  const manifests = [
    "vize_atelier_core",
    "vize_atelier_dom",
    "vize_atelier_jsx",
    "vize_atelier_sfc",
    "vize_atelier_ssr",
    "vize_atelier_vapor",
    "vize_canon",
    "vize_croquis",
    "vize_l1",
    "vize_l1_to_l2",
    "vize_patina",
  ];
  for (const name of manifests) {
    assert.match(
      read(`${name.startsWith("vize_l") ? "davinci" : "crates"}/${name}/Cargo.toml`),
      new RegExp(`^${publishedDifferentialAlias} = \\["legacy-differential"\\]$`, "m"),
      `${name} lost its published feature alias`,
    );
  }
  assert.match(
    read("crates/vize_atelier_sfc/Cargo.toml"),
    new RegExp(`^${publishedDomAlias} = \\["legacy-dom-differential"\\]$`, "m"),
  );
});
