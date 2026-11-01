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
  assert.match(block, /uses: pnpm\/action-setup@[a-f0-9]{40}(?=\s|$)/u);
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

function requireOriginalOrder(consumer: string): void {
  assert.match(
    consumer,
    /node-version: \$\{\{ inputs\.role == 'runtime-prop-null-11252' && '26\.12\.1' \|\| '24\.22\.1' \}\}/u,
  );
  assert.match(consumer, /package-manager-cache: false/u);
  const actions = [
    /uses: actions\/setup-node@[a-f0-9]{40}(?=\s|$)/u,
    /uses: pnpm\/action-setup@[a-f0-9]{40}(?=\s|$)/u,
  ].map((pattern) => pattern.exec(consumer)?.index ?? -1);
  const commands = [
    'root="$RUNNER_TEMP/human-reports-$ROLE-$PHASE"',
    'pnpm_cli="$(node -p',
    "human_reports_11252_11253_public/bootstrap.ts",
    "human_reports_11252_11253_public/run.ts",
    '--execute-public "$root-request/request.json" "$pnpm_cli" "$root-output"',
  ].map((value) => consumer.indexOf(value));
  const ordered = [...actions, ...commands];
  assert(
    ordered.every(
      (position, index) => position >= 0 && (index === 0 || position > ordered[index - 1]!),
    ),
  );
}

test("original setup order permits full-pin rotation but refuses unpinned or reordered setup", () => {
  const consumer = workflow.slice(workflow.indexOf("  public-consumer:"));
  requireOriginalOrder(consumer);
  requireOriginalOrder(consumer.replace(/[a-f0-9]{40}/gu, "b".repeat(40)));
  assert.throws(() =>
    requireOriginalOrder(
      consumer.replace(/actions\/setup-node@[a-f0-9]{40}/u, "actions/setup-node@v6"),
    ),
  );
  assert.throws(() =>
    requireOriginalOrder(
      consumer.replace(
        /uses: (actions\/setup-node|pnpm\/action-setup)@/gu,
        (_full, name: string) =>
          `uses: ${name === "actions/setup-node" ? "pnpm/action-setup" : "actions/setup-node"}@`,
      ),
    ),
  );
});

test("public checkout keeps no persisted credential while API bootstrap retains its scoped token", () => {
  const consumer = workflow.slice(workflow.indexOf("  public-consumer:"));
  const requireNoCredential = (source: string) => {
    const checkout = source.match(
      /- uses: actions\/checkout@[a-f0-9]{40}[^]*?(?=\n      - )/u,
    )?.[0];
    assert(checkout);
    assert.match(checkout, /persist-credentials:\s*false/u);
    assert.match(source, /GITHUB_TOKEN:\s*\$\{\{ github.token \}\}/u);
  };
  requireNoCredential(consumer);
  assert.throws(() =>
    requireNoCredential(
      consumer.replace("persist-credentials: false", "persist-credentials: true"),
    ),
  );
});

test("local and remote controller custody include bootstrap and source-owned input data", async () => {
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
  for (const name of ["dependency-bin-policy.json.txt", "pnpm-shim-templates.json.txt"])
    assert(
      closure.some(
        (file) => file.path === `tools/support/release/human_reports_11252_11253_public/${name}`,
      ),
      "non-imported policy/template data must be an explicit remote source seed",
    );
  assert(
    closure.some(
      (file) =>
        file.path ===
        "tools/support/release/human_reports_11252_11253_public/after-input-plan.json.txt",
    ),
  );
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
