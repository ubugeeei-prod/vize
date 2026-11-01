import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { controllerFiles } from "../../../tools/support/release/human_reports_11252_11253_public/controller.ts";
import { remoteControllerClosure } from "../../../tools/support/release/human_reports_11252_11253_public/controller-authority.ts";
import { recipes } from "../../../tools/support/release/human_reports_11252_11253_public/recipes.ts";
import { gitBlob } from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const bootstrapPath = "tools/support/release/human_reports_11252_11253_public/pnpm-bootstrap.json";
const workflowPath = ".github/workflows/published-human-reports-11252-11253.yml";
const workflow = readFileSync(resolve(root, workflowPath), "utf8");
const actionBlock = workflow.match(/      - uses: pnpm\/action-setup@[^]*?(?=\n      - )/u)?.[0];

function requireIsolatedSetup(block: string | undefined, manifest: unknown): void {
  assert(block, "pinned pnpm bootstrap action required");
  assert.match(block, /version: "12\.16\.1"/u);
  assert.match(block, /run_install: false/u);
  const selectedPath = block.match(/package_json_file: ([^\s]+)/u)?.[1];
  assert.equal(selectedPath, bootstrapPath, "repository packageManager must not select bootstrap");
  assert.deepEqual(manifest, { private: true, packageManager: "pnpm@12.16.1" });
  for (const recipe of Object.values(recipes)) assert.equal(recipe.pnpm, "12.16.1");
}

test("reporter pnpm bootstrap refuses repository-manifest fallback and conflicting versions", () => {
  const manifest: unknown = JSON.parse(readFileSync(resolve(root, bootstrapPath), "utf8"));
  requireIsolatedSetup(actionBlock, manifest);
  assert.throws(() =>
    requireIsolatedSetup(actionBlock?.replace(bootstrapPath, "package.json"), manifest),
  );
  assert.throws(() =>
    requireIsolatedSetup(actionBlock, { private: true, packageManager: "pnpm@12.1.0" }),
  );
  assert.throws(() => requireIsolatedSetup(actionBlock?.replace("12.16.1", "12.1.0"), manifest));
});

test("original host Node and isolated action setup precede executable capture and recipe execution", () => {
  const consumer = workflow.slice(workflow.indexOf("  public-consumer:"));
  assert.match(
    consumer,
    /node-version: \$\{\{ inputs\.role == 'runtime-prop-null-11252' && '26\.12\.1' \|\| '24\.22\.1' \}\}/u,
  );
  assert.match(consumer, /package-manager-cache: false/u);
  const ordered = [
    "uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38",
    "uses: pnpm/action-setup@ea17c68df8912ef543352723c149a84f56e3d413",
    'root="$RUNNER_TEMP/human-reports-$ROLE-$PHASE"',
    'pnpm_cli="$(node -p',
    "human_reports_11252_11253_public/bootstrap.ts",
    "human_reports_11252_11253_public/run.ts",
    '--execute-public "$root-request/request.json" "$pnpm_cli" "$root-output"',
  ].map((value) => consumer.indexOf(value));
  assert(
    ordered.every(
      (position, index) => position >= 0 && (index === 0 || position > ordered[index - 1]!),
    ),
  );
});

test("local and independently reconstructed controller custody both include bootstrap manifest", async () => {
  const head = "a".repeat(40);
  const closure = await remoteControllerClosure(head, async (request) => {
    assert(request.startsWith("contents/") && request.endsWith(`?ref=${head}`));
    const path = request.slice("contents/".length, -`?ref=${head}`.length);
    const raw = readFileSync(resolve(root, path));
    return {
      type: "file",
      path,
      encoding: "base64",
      content: raw.toString("base64"),
      sha: gitBlob(raw),
    };
  });
  assert(closure.some((file) => file.path === bootstrapPath));
  const actual = controllerFiles().map((file) => ({
    path: file.path.slice(root.length),
    bytes: file.bytes,
    sha256: file.sha256,
    gitBlob: gitBlob(readFileSync(file.path)),
  }));
  assert.deepEqual(
    actual.sort((a, b) => a.path.localeCompare(b.path)),
    closure,
  );
});
