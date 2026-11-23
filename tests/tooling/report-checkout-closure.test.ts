// The serial PR report gates check out only their script closure (#6830).
// A full checkout cost 13–14 s per report on the PR critical path.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { toolingProgram } from "./support/program-path.ts";
import { parse } from "yaml";
import { localImportInputs } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
type Step = { uses?: string; run?: string; with?: Record<string, unknown> };
const reports = [
  [".github/workflows/pr-rust-products.yml", "rust-product-report"],
  [".github/workflows/pr-rust-checks.yml", "rust-source-report"],
  [".github/workflows/pr-source-checks.yml", "source-report"],
] as const;

for (const [file, id] of reports) {
  await test(`${id} checks out exactly its gate's runtime closure`, (t) => {
    const steps: Step[] = parse(readFileSync(join(root, file), "utf8")).jobs[id].steps;
    const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"));
    assert.equal(checkout?.with?.["sparse-checkout-cone-mode"], false);
    assert.equal(checkout?.with?.["fetch-depth"], 1);
    const sparse = String(checkout?.with?.["sparse-checkout"]).trim().split("\n");
    const scripts = steps.flatMap((step) =>
      [...(step.run ?? "").matchAll(/\bnode (tools\/\S+\.(?:ts|mjs))/g)].map((match) => match[1]),
    );
    assert.equal(scripts.length, 1, "one gate script");
    const closure = localImportInputs(scripts[0], root, new Set(), true) as {
      complete: boolean;
      files: string[];
    };
    assert.equal(closure.complete, true, "literal import closure");
    const files = [...new Set(closure.files)].filter((path) => existsSync(join(root, path)));
    const actions = steps
      .filter((step) => step.uses?.startsWith("./"))
      .map((step) => `/${step.uses!.slice(2)}/`);
    assert.deepEqual(
      [...sparse].sort(),
      ["/package.json", ...actions, ...files.map((path) => `/${path}`)].sort(),
    );

    // The gate module loads from a tree holding only the sparse paths.
    const scratch = mkdtempSync(join(tmpdir(), "vize-report-closure-"));
    t.after(() => rmSync(scratch, { recursive: true, force: true }));
    for (const entry of sparse) {
      const path = entry.replace(/^\/|\/$/g, "");
      mkdirSync(dirname(join(scratch, path)), { recursive: true });
      cpSync(join(root, path), join(scratch, path), { recursive: true });
    }
    const entry = pathToFileURL(join(scratch, scripts[0])).href;
    execFileSync(process.execPath, [toolingProgram("process/import-selected-module.js"), entry], {
      cwd: scratch,
      stdio: "pipe",
    });
  });
}
