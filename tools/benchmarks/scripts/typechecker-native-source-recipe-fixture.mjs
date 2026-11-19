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
  "e9bc232282eddeda1e237c20bff4a158e1ff98a4d6e217ef635d741643cb498c",
);
export const bomRequiredRun = readFileSync(
  new URL("../../../tests/_fixtures/tooling/native-current-recipe-8253.sh", import.meta.url),
  "utf8",
);
assert.equal(
  hash(bomRequiredRun),
  "347fb42ea486692c1760fb5771a7aa04d13d276c544dfc2e986af9418df15384",
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
  "623681241ea1fbcdd18bb28f4bb1fea60a3fd514a1f049e20090fa36ef96cca7",
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
  "a37819a3c934a65eeb6ec3fcd7e398942c1a09095a92e4365dcd8384826bb6fc",
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
  "686d855f4ea73f2e4389f1c973a5c921b96030a43e745188b22c614d385d98bf",
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

export const passiveRequiredRun = readFileSync(
  new URL(
    "../../../tests/_fixtures/tooling/native-current-recipe-passive-lsp-3952.sh",
    import.meta.url,
  ),
  "utf8",
);
assert.equal(
  hash(passiveRequiredRun),
  "5d0e6b8191bd85d94e40ab1a4c7ebbd67545c025828bf32c8b9818b17a7652eb",
);
const selectedAliasFirstCargo = selectedAliasRequiredRun.split("\n")[3];
const passiveCargo =
  "env -u VIZE_INLAY_HINT_CAPTURE VIZE_LSP_PASSIVE_EVIDENCE=\"$RUNNER_TEMP/lsp-passive-3952\" VIZE_TRACE_EDITOR_PREPARATION=1 cargo test --locked --profile ci-opt -p vize --test lsp_passive_evidence_cli --test lsp_reactive_diagnostics_cli --config 'profile.ci-opt.inherits=\"release\"' --config 'profile.ci-opt.lto=\"thin\"' --config 'profile.ci-opt.codegen-units=16' --config 'profile.ci-opt.package.vize.strip=\"symbols\"' -- --nocapture";
assert.equal(
  passiveRequiredRun,
  selectedAliasRequiredRun.replace(
    selectedAliasFirstCargo + "\n",
    selectedAliasFirstCargo + "\n" + passiveCargo + "\n",
  ),
);

// The CLI's integration tests became one binary: each former `--test` target
// of the vize package is now a module filter of `--test integration`.
export function integrationBinaryRun(run) {
  return run
    .split("\n")
    .map((line) => {
      if (!line.includes(" -p vize --test ")) return line;
      const filters = [...line.matchAll(/ --test ([a-z0-9_]+)/gu)].map(([, name]) => `${name}::`);
      let first = true;
      const command = line.replace(/ --test ([a-z0-9_]+)/gu, () => {
        if (!first) return "";
        first = false;
        return " --test integration";
      });
      return command.replace(" -- --nocapture", ` -- ${filters.join(" ")} --nocapture`);
    })
    .join("\n");
}
export const currentRequiredRun = readFileSync(
  new URL(
    "../../../tests/_fixtures/tooling/native-current-recipe-integration-binary.sh",
    import.meta.url,
  ),
  "utf8",
);
assert.equal(
  hash(currentRequiredRun),
  "c2bedaf6133d65c459b0bd868786d794ce3cdb2f817769ceb22da456fdb9a2a4",
);
assert.equal(currentRequiredRun, integrationBinaryRun(passiveRequiredRun));

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
  let pending = false;
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
    const result = run({
      root,
      cut,
      source,
      pin,
      env,
      event: { pull_request: pr },
      permission: { role_name: "maintain" },
      parse,
    });
    if (result?.then) {
      pending = true;
      return result.finally(() => rmSync(root, { recursive: true, force: true }));
    }
    return result;
  } finally {
    if (!pending) rmSync(root, { recursive: true, force: true });
  }
}
