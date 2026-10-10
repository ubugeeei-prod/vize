/** Full immutable source metadata plus disposable real-Git custody fixtures. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import { dirname, join } from "node:path";
import { gunzipSync } from "node:zlib";
import { WORKFLOW } from "./typechecker-native-source-recipe.mjs";
const require = createRequire(import.meta.url);
export const { parse } = require("yaml");
export const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const archive = readFileSync(
  new URL("../../../tests/_fixtures/tooling/native-source-recipe-436.json.gz", import.meta.url),
);
assert.equal(hash(archive), "b52db3ad5407614c9fd985a28f22083808d9d17852583ce6fd57153b34164cf0");
const raw = gunzipSync(archive);
assert.equal(hash(raw), "2b98c5e519c8584a3bc625210c03ce6ea7611ab973f871012eb2510222e52976");
export const frozen = JSON.parse(raw);
export const currentBytes = readFileSync(
  new URL("../../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
  "utf8",
);
export const currentOriginalRun = readFileSync(
  new URL("../../../tests/_fixtures/tooling/native-current-recipe-8231.sh", import.meta.url),
  "utf8",
);
assert.equal(
  hash(currentOriginalRun),
  "397a7119bdeab7cecdb7e1f7c5ed62b8df1232005fcd3d538ac6f20bf761fb13",
);
export const bomRequiredRun = readFileSync(
  new URL("../../../tests/_fixtures/tooling/native-current-recipe-8253.sh", import.meta.url),
  "utf8",
);
assert.equal(
  hash(bomRequiredRun),
  "b2bd7c08e90ca1625044d09cbe221c22f8a8344be8c4347854acb73a66dfc05f",
);
assert.equal(
  bomRequiredRun,
  currentOriginalRun.replace(
    " --test lsp_bare_script_symbols_cli --config",
    " --test lsp_bare_script_symbols_cli --test check_tsconfig_bom_cli --config",
  ),
);
export const ignoreRequiredRun = readFileSync(
  new URL("../../../tests/_fixtures/tooling/native-current-recipe-ignore-3984.sh", import.meta.url),
  "utf8",
);
assert.equal(
  hash(ignoreRequiredRun),
  "8a2b3e2ffc52c4a80a3c02ae15cede5d8c15fd3415a2368aa1974135703e9c56",
);
assert.equal(
  ignoreRequiredRun,
  bomRequiredRun.replace(
    " --test check_tsconfig_bom_cli --config",
    " --test check_tsconfig_bom_cli --test check_tsconfig_ignore_cli --config",
  ),
);

export const jsxRequiredRun = readFileSync(
  new URL(
    "../../../tests/_fixtures/tooling/native-current-recipe-jsx-slot-8419.sh",
    import.meta.url,
  ),
  "utf8",
);
assert.equal(
  hash(jsxRequiredRun),
  "3e1cbab6ccc79028c9de0d086801c140b0e99f1876dc9b7002acb74144b600e7",
);
assert.equal(
  jsxRequiredRun,
  ignoreRequiredRun.replace(
    " --test check_tsconfig_ignore_cli --config",
    " --test check_tsconfig_ignore_cli --test check_jsx_slot_parameter_types_cli --config",
  ),
);

export const selectedAliasRequiredRun = readFileSync(
  new URL(
    "../../../tests/_fixtures/tooling/native-current-recipe-selected-alias-3984.sh",
    import.meta.url,
  ),
  "utf8",
);
assert.equal(
  hash(selectedAliasRequiredRun),
  "ae22cd812c7def2272acf9596ba7c9fe0e94d2c590b5f9d3411fad73b84b14a2",
);
const receiptCommands = [
  "node tools/benchmarks/scripts/typechecker-template-emit-projection-receipt.mjs",
  "node tools/benchmarks/scripts/typechecker-native-template-emit-receipt.mjs",
  "node tools/benchmarks/scripts/typechecker-native-vue-helper-receipt.mjs",
];
assert.equal(
  selectedAliasRequiredRun,
  jsxRequiredRun
    .replace(
      " --test check_jsx_slot_parameter_types_cli --config",
      " --test check_jsx_slot_parameter_types_cli --test check_canon_selected_alias_cli --config",
    )
    .replace(receiptCommands.join("\n") + "\n", receiptCommands.join("; ") + "\n"),
);

export const currentRequiredRun = readFileSync(
  new URL(
    "../../../tests/_fixtures/tooling/native-current-recipe-passive-lsp-3952.sh",
    import.meta.url,
  ),
  "utf8",
);
assert.equal(
  hash(currentRequiredRun),
  "9a6c630b68910f0c48f45040eb166ae7fa7fa4b0df798d34c74154eb1742d518",
);
const selectedAliasFirstCargo = selectedAliasRequiredRun.split("\n")[3];
const passiveCargo =
  "env -u VIZE_INLAY_HINT_CAPTURE VIZE_LSP_PASSIVE_EVIDENCE=\"$RUNNER_TEMP/lsp-passive-3952\" VIZE_TRACE_EDITOR_PREPARATION=1 cargo test --locked --profile ci-opt -p vize --test lsp_passive_evidence_cli --test lsp_reactive_diagnostics_cli --config 'profile.ci-opt.inherits=\"release\"' --config 'profile.ci-opt.lto=\"thin\"' --config 'profile.ci-opt.codegen-units=16' --config 'profile.ci-opt.package.vize.strip=\"symbols\"' -- --nocapture";
assert.equal(
  currentRequiredRun,
  selectedAliasRequiredRun.replace(
    selectedAliasFirstCargo + "\n",
    selectedAliasFirstCargo + "\n" + passiveCargo + "\n",
  ),
);

export function git(root, ...args) {
  const r = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(r.error, undefined);
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
}
export function put(root, path, text) {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), text);
}
export function commit(root) {
  git(root, "add", "-A");
  git(root, "commit", "-m", "authored custody control");
  return git(root, "rev-parse", "HEAD");
}
export function fixture(run, { workflow } = {}) {
  const root = mkdtempSync(join(os.tmpdir(), "native-source-recipe-"));
  try {
    git(root, "init", "--initial-branch=main");
    git(root, "config", "user.name", "Recipe control");
    git(root, "config", "user.email", "recipe@example.invalid");
    git(root, "config", "commit.gpgsign", "false");
    for (const row of frozen.files) {
      assert.equal(hash(row.before), row.beforeSha256);
      assert.equal(hash(row.after), row.afterSha256);
      put(root, row.path, row.before);
    }
    put(root, "crates/vize/src/recipe-control.rs", "// unchanged production control\n");
    if (workflow !== undefined) put(root, WORKFLOW, workflow);
    const cut = commit(root);
    for (const row of frozen.files)
      put(root, row.path, row.path === WORKFLOW && workflow !== undefined ? workflow : row.after);
    const source = commit(root);
    const pr = {
      number: 8249,
      state: "open",
      draft: true,
      merged: false,
      user: { login: "maintainer" },
      base: { ref: "main", sha: cut, repo: { full_name: "ubugeeei-prod/vize" } },
      head: { sha: source, ref: "release/v0.436.0", repo: { full_name: "ubugeeei-prod/vize" } },
      body: [
        `vize-release-pin: immutable-v1`,
        `vize-release-pin-head: ${source}`,
        `vize-release-pin-cut: ${cut}`,
        "vize-release-base-version: 0.435.1",
        "vize-release-integration: 8250",
      ]
        .map((x) => `<!-- ${x} -->`)
        .join("\n"),
    };
    const message = `Vize immutable release cut\n\nSource-PR: #8249\nSource-head: ${source}\nSource-cut: ${cut}\nTag: v0.436.0\nBase-version: 0.435.1\nIntegration-PR: #8250`;
    const pin = git(
      root,
      "commit-tree",
      git(root, "rev-parse", `${source}^{tree}`),
      "-p",
      source,
      "-m",
      message,
    );
    const env = {
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_EVENT_NAME: "pull_request",
      SOURCE_SHA: source,
      DRIVER_SHA: source,
      MAIN_SOURCE_SHA: cut,
      MAIN_HEAD_SHA: cut,
      GITHUB_WORKFLOW_SHA: source,
      GITHUB_WORKFLOW_REF: `ubugeeei-prod/vize/${WORKFLOW}@refs/pull/8249/merge`,
    };
    run({
      root,
      cut,
      source,
      pin,
      env,
      event: { pull_request: pr },
      permission: { role_name: "maintain" },
      parse,
    });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
