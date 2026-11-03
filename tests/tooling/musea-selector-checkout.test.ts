import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../../", import.meta.url));
const job = parse(readFileSync(join(root, ".github/workflows/musea-browser.yml"), "utf8")).jobs
  .plan;
const inputs = job.steps.find((step: { uses?: string }) =>
  step.uses?.startsWith("actions/checkout@"),
).with;
const command = job.steps.find((step: { id?: string }) => step.id === "plan").run;
const planner = "tools/support/compat/github/plan-product-integrations.ts";
const runtime = ["package.json", planner].sort();

function put(cwd: string, file: string, text: string) {
  mkdirSync(dirname(join(cwd, file)), { recursive: true });
  writeFileSync(join(cwd, file), text);
}
function git(cwd: string, ...args: string[]) {
  return execFileSync("git", args, { cwd, encoding: "utf8", stdio: "pipe" }).trim();
}
function files(cwd: string, prefix = ""): string[] {
  return readdirSync(join(cwd, prefix), { withFileTypes: true })
    .filter((entry) => entry.name !== ".git")
    .flatMap((entry) => {
      const name = prefix ? `${prefix}/${entry.name}` : entry.name;
      return entry.isDirectory() ? files(cwd, name) : [name];
    })
    .sort();
}
function assertOutputs(output: string | null, expected: Record<string, string>) {
  assert.notEqual(output, null);
  const pairs = output!
    .split("\n")
    .filter((line) => line !== "")
    .map((line) => {
      const match = /^([a-z][a-z-]*)=(true|false)$/.exec(line);
      assert.ok(match, `invalid output pair: ${line}`);
      return [match[1], match[2]];
    });
  assert.equal(new Set(pairs.map(([key]) => key)).size, pairs.length);
  assert.deepEqual(Object.fromEntries(pairs), expected);
}

test("selector output expectations keep exact keys and values without imposing order", () => {
  const expected = { nuxt: "true", "musea-native": "false", "musea-browser": "true" };
  const packet = "musea-browser=true\nnuxt=true\nmusea-native=false\n";
  assertOutputs(packet, expected);
  for (const invalid of [
    packet + "nuxt=true\n",
    packet + "unknown=true\n",
    packet.replace("nuxt=true\n", ""),
    packet.replace("nuxt=true\n", "nuxt=true \n"),
    packet.replace("nuxt=true\n", "nuxt=true\r\n"),
  ])
    assert.throws(() => assertOutputs(invalid, expected));
});

await test("Musea selection preserves its Git inputs without materializing product sources", async (t) => {
  const scratch = mkdtempSync(join(tmpdir(), "vize-musea-checkout-"));
  t.after(() => rmSync(scratch, { recursive: true, force: true }));
  const origin = join(scratch, "origin");
  mkdirSync(origin);
  git(origin, "init", "-q");
  for (const [key, value] of [
    ["user.name", "CI fixture"],
    ["user.email", "ci@example.invalid"],
    ["commit.gpgsign", "false"],
    ["core.hooksPath", "/dev/null"],
    ["uploadpack.allowFilter", "true"],
  ])
    git(origin, "config", key, value);
  for (const file of runtime) put(origin, file, readFileSync(join(root, file), "utf8"));
  const removed = "npm/builder/vite-musea/src/removed.ts";
  const added = "npm/framework/nuxt/src/added.ts";
  put(origin, removed, "removed Musea product bytes\n");
  put(origin, "crates/removed/src/lib.rs", "removed native product bytes\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "runtime and original product inputs");
  const base = git(origin, "rev-parse", "HEAD");
  rmSync(join(origin, removed));
  rmSync(join(origin, "crates/removed/src/lib.rs"));
  put(origin, added, "added Nuxt product bytes\n");
  git(origin, "add", "-A");
  git(origin, "commit", "-qm", "add and delete product inputs outside runtime");
  const productHead = git(origin, "rev-parse", "HEAD");
  put(origin, "docs/unrelated.md", "unrelated prose\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "unrelated documentation input");
  const head = git(origin, "rev-parse", "HEAD");
  const browserOnly = "tools/support/release/public_acceptance/browser_observation.ts";
  put(origin, browserOnly, "browser-only integration input\n");
  git(origin, "add", ".");
  git(origin, "commit", "-qm", "one browser integration changes");
  const browserHead = git(origin, "rev-parse", "HEAD");

  function checkout(name: string, configured: boolean, revision = head) {
    const cwd = join(scratch, name);
    mkdirSync(cwd);
    git(cwd, "init", "-q");
    git(cwd, "remote", "add", "origin", pathToFileURL(origin).href);
    const sparse = configured ? inputs["sparse-checkout"] : undefined;
    assert.equal(inputs["fetch-depth"] ?? 1, 1);
    assert.equal(inputs["persist-credentials"], false);
    git(
      cwd,
      "fetch",
      "--no-tags",
      "--depth=1",
      ...(sparse ? ["--filter=blob:none"] : []),
      "origin",
      revision,
    );
    if (sparse) {
      assert.equal(inputs["sparse-checkout-cone-mode"], false);
      git(cwd, "config", "core.sparseCheckout", "true");
      put(cwd, ".git/info/sparse-checkout", `${sparse.trim()}\n`);
    }
    git(cwd, "checkout", "-q", "--detach", "FETCH_HEAD");
    return cwd;
  }
  const full = checkout("full", false),
    selected = checkout("selected", true);
  assert.equal(job["timeout-minutes"], 3);
  assert.equal(git(selected, "rev-parse", "--is-shallow-repository"), "true");
  const present = (cwd: string, object: string) => {
    const result = spawnSync("git", ["cat-file", "-e", object], {
      cwd,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      stdio: "pipe",
    });
    assert.ifError(result.error);
    assert.notEqual(result.status, null);
    return result.status === 0;
  };
  assert.equal(present(selected, base), false, "the actual command must fetch the missing base");
  function execute(cwd: string, event: string, comparison = base, target = head) {
    const output = join(scratch, `${basename(cwd)}-outputs`);
    rmSync(output, { force: true });
    const result = spawnSync("bash", ["-e", "-o", "pipefail", "-c", command], {
      cwd,
      env: {
        ...process.env,
        GITHUB_EVENT_NAME: event,
        BASE_SHA: comparison,
        GITHUB_SHA: target,
        GITHUB_OUTPUT: output,
      },
      encoding: "utf8",
    });
    return {
      code: result.status,
      stdout: result.stdout,
      stderr: result.stderr,
      outputs: existsSync(output) ? readFileSync(output, "utf8") : null,
    };
  }
  const all = { nuxt: "true", "musea-native": "true", "musea-browser": "true" };
  for (const event of ["pull_request", "merge_group", "push", "schedule", "workflow_dispatch"])
    await t.test(
      `${event} retains complete selector output after fetching outside-runtime changes`,
      () => {
        const actual = execute(selected, event),
          reference = execute(full, event);
        assert.equal(actual.code, 0, actual.stderr);
        assert.equal(reference.code, 0, reference.stderr);
        assertOutputs(actual.outputs, all);
        assert.equal(actual.outputs, reference.outputs);
        assert.equal(actual.stdout, reference.stdout);
      },
    );
  await t.test("unrelated prose preserves the complete negative selection", () => {
    const actual = execute(selected, "pull_request", productHead),
      reference = execute(full, "pull_request", productHead);
    assert.equal(actual.code, 0, actual.stderr);
    assertOutputs(actual.outputs, {
      nuxt: "false",
      "musea-native": "false",
      "musea-browser": "false",
    });
    assert.equal(actual.outputs, reference.outputs);
    assert.equal(actual.stdout, reference.stdout);
  });
  await t.test("a browser-only change leaves Nuxt and native Musea unselected", () => {
    const browserFull = checkout("browser-full", false, browserHead);
    const browserSelected = checkout("browser-selected", true, browserHead);
    const actual = execute(browserSelected, "pull_request", head, browserHead);
    const reference = execute(browserFull, "pull_request", head, browserHead);
    assert.equal(actual.code, 0, actual.stderr);
    assert.equal(reference.code, 0, reference.stderr);
    assertOutputs(actual.outputs, {
      nuxt: "false",
      "musea-native": "false",
      "musea-browser": "true",
    });
    assert.equal(actual.outputs, reference.outputs);
    assert.equal(actual.stdout, reference.stdout);
    assert.deepEqual(files(browserSelected), runtime);
    assert.equal(present(browserFull, `${browserHead}:${browserOnly}`), true);
    assert.equal(present(browserSelected, `${browserHead}:${browserOnly}`), false);
  });
  for (const [name, event, comparison, target] of [
    ["zero base", "push", "0".repeat(40), head],
    ["invalid base", "pull_request", "invalid", head],
    ["invalid head", "merge_group", base, "invalid"],
    ["unsupported event", "unknown", base, head],
  ])
    await t.test(`${name} retains the original success or refusal contract`, () => {
      const actual = execute(selected, event, comparison, target),
        reference = execute(full, event, comparison, target);
      assert.equal(actual.code, reference.code);
      assert.equal(actual.code, name === "zero base" ? 0 : 1);
      if (name === "zero base") assertOutputs(actual.outputs, all);
      else assert.equal(actual.outputs, null);
      assert.equal(actual.outputs, reference.outputs);
      assert.equal(actual.stdout, reference.stdout);
    });
  await t.test(
    "configured checkout contains only the runtime and keeps fetched product blobs absent",
    () => {
      assert.deepEqual(files(selected), runtime);
      assert.equal(present(full, `${head}:${added}`), true);
      for (const object of [`${head}:${added}`, `${base}:${removed}`])
        assert.equal(present(selected, object), false);
    },
  );
  await t.test("a missing selector refuses to publish outputs", () => {
    rmSync(join(selected, planner));
    const actual = execute(selected, "pull_request");
    assert.equal(actual.code, 1);
    assert.equal(actual.outputs, null);
    assert.match(actual.stderr, /MODULE_NOT_FOUND/);
  });
});
