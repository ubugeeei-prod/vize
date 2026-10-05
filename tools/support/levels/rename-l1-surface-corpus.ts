import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const oldPath = "davinci/vize_l1/tests/davinci_surface_corpus.rs";
const newPath = "davinci/vize_l1/tests/surface_corpus.rs";
const phase = process.argv[2];
if (!["moves", "integrate", "check"].includes(phase)) {
  throw new Error("Usage: rename-l1-surface-corpus.ts moves|integrate|check");
}
const oldExists = existsSync(resolve(root, oldPath));
const newExists = existsSync(resolve(root, newPath));
if (oldExists === newExists) throw new Error("Expected exactly one surface corpus target path");
if (phase === "moves") {
  if (oldExists) execFileSync("git", ["mv", oldPath, newPath], { cwd: root });
} else {
  if (oldExists) throw new Error("Move the surface corpus target before integrating");
  const updates = new Map<string, string>();
  function replace(path: string, before: string, after: string) {
    const source = updates.get(path) ?? readFileSync(resolve(root, path), "utf8");
    const oldCount = source.split(before).length - 1;
    const newCount = source.split(after).length - 1;
    if (oldCount === 1 && newCount === 0) updates.set(path, source.replace(before, after));
    else if (newCount === 1 && oldCount === after.split(before).length - 1)
      updates.set(path, source);
    else throw new Error(`Unexpected surface corpus integration anchor: ${path}`);
  }
  replace(
    "davinci/vize_l1/Cargo.toml",
    "tests/davinci_surface_corpus.rs",
    "tests/surface_corpus.rs",
  );
  replace(
    "davinci/vize_l1/Cargo.toml",
    'name = "davinci_surface_corpus"',
    'name = "surface_corpus"',
  );
  replace(newPath, "--test davinci_surface_corpus", "--test surface_corpus");
  replace(
    "tests/davinci_test_support/src/surface_fixture.rs",
    "(`davinci_surface_corpus.rs`)",
    "(`surface_corpus.rs`)",
  );
  replace(
    "docs/davinci/plan/phase-2-records.md",
    "[`davinci_surface_corpus`](../../../davinci/vize_l1/tests/davinci_surface_corpus.rs)",
    "[`surface_corpus`](../../../davinci/vize_l1/tests/surface_corpus.rs)",
  );
  const command =
    "cargo test -p vize_l1 --features legacy-differential --test surface_corpus -- --nocapture";
  const prAnchor = "      - name: Test affected native Program and Vue navigation\n";
  const step = `      - name: Test L1 feature-enabled surface corpus\n        if: \u0024{{ contains(fromJSON(inputs.rust-plan).packages, 'vize_l1') }}\n        run: ${command}\n`;
  replace(".github/workflows/pr-rust-checks.yml", prAnchor, step + prAnchor);
  const first =
    "        cargo test -p vize_l1_to_l2 --features legacy-differential --test davinci_lowering_corpus -- --nocapture\n";
  replace(
    ".github/actions/test-rust-workspace-differential/action.yml",
    first,
    `        ${command}\n` + first,
  );
  const gate = "tests/tooling/github-workflows-merge-queue.test.ts";
  replace(gate, "const tailCommands = [\n", `const tailCommands = [\n  "${command}",\n`);
  replace(
    gate,
    '["davinci_remarks_corpus", 3, 42]',
    '["surface_corpus", 1, 42],\n      ["davinci_remarks_corpus", 4, 42]',
  );
  replace(gate, '["no-such-command", 11, 0]', '["no-such-command", 12, 0]');
  replace(gate, "lines[5].endsWith", "lines[6].endsWith");
  replace(gate, "lines[10].endsWith", "lines[11].endsWith");
  const canonical = "docs/davinci/decisions/2026-09-27-level-restructure.md";
  const anchor = "the remaining tooling names stay open.";
  const decision =
    " [L1 surface corpus naming](./2026-10-04-l1-surface-corpus-name.md) moves one feature-gated target with its original byte custody, preserves published aliases and historical witnesses, and requires actual feature-enabled PR and protected-queue execution of the unchanged 42-case battery without native product or history-gate credit.";
  replace(canonical, anchor, anchor + decision);
  // Buffer all replacements so a drifted later anchor cannot leave partial integration.
  for (const [path, changed] of updates) {
    if (readFileSync(resolve(root, path), "utf8") !== changed) {
      if (phase === "check") throw new Error(`Unintegrated surface corpus reference: ${path}`);
      writeFileSync(resolve(root, path), changed);
    }
  }
}
