import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = fileURLToPath(new URL("../..", import.meta.url));

function read(path: string): string {
  return readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
}

test("differential feature rename is complete in tracked source", () => {
  const output = execFileSync(
    "python3",
    ["tools/support/levels/rename-differential-features.py", "--check"],
    { cwd: root, encoding: "utf8" },
  );
  assert.match(output, /^0 files contain old selectors$/m);
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
