import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const baseline = "fc745ea5a2c9efcf6b1549d6312c61d4f2770697";
const schema = "vize.js-install-closure.source-recipe.v2";
const installerPath = "tools/support/compat/github/install-js-checks.ts";
export const historicalInstaller = Object.freeze({
  revision: baseline,
  path: installerPath,
  gitBlob: "bd418ef0a2e2a0a9e66a7c0e7281a599f28afc60",
  sha256: "d5cb222d10bf8d0e40d2a832c9b430364b1f1525b20af2667017fef6c959a763",
  bytes: 1625,
});
const historicalFile = new URL(
  "../../../../tests/fixtures/ci/js-install-closure/install-js-checks.fc745.ts.txt",
  import.meta.url,
);
const compareNames = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
const observer = resolve(dirname(fileURLToPath(import.meta.url)), "../../../..");
const inputs = [
  "package.json",
  "pnpm-workspace.yaml",
  "pnpm-lock.yaml",
  "npm/builder/shared/package.json",
  "npm/builder/shared/mapped-module.ts",
  "npm/builder/rspack/package.json",
  "npm/builder/rspack/src/shared/source-map.test.ts",
];
const digest = (value) => createHash("sha256").update(value).digest("hex");
const hashes = (root) =>
  Object.fromEntries(inputs.map((file) => [file, digest(readFileSync(resolve(root, file)))]));
const git = (root, args) => execFileSync("git", args, { cwd: root, maxBuffer: 64 * 1024 * 1024 });
const gitText = (root, args) => git(root, args).toString("utf8").trim();
const recipeIdentity = (body, revision) => ({
  revision,
  path: installerPath,
  gitBlob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
  sha256: digest(body),
  bytes: body.length,
});

export function currentInstallSource(root = observer) {
  const revision = gitText(root, ["rev-parse", "HEAD"]);
  const source = { revision, tree: gitText(root, ["rev-parse", "HEAD^{tree}"]) };
  const body = git(root, ["show", `${revision}:${installerPath}`]);
  const installer = recipeIdentity(body, revision);
  assert.equal(installer.gitBlob, gitText(root, ["rev-parse", `${revision}:${installerPath}`]));
  const expectedInputs = Object.fromEntries(
    inputs.map((file) => [file, digest(git(root, ["show", `${revision}:${file}`]))]),
  );
  return { source, inputs: expectedInputs, installer };
}

export function inspectInstallRecipe(role, root, authority) {
  assert.ok(["before", "after"].includes(role), "declared installer role");
  assert.deepEqual(currentInstallSource(root), authority, "actual current source authority");
  assert.deepEqual(hashes(root), authority.inputs, "actual current source inputs");
  assert.equal(
    gitText(root, ["ls-files", "--others", "--exclude-standard"]),
    "",
    "untracked source change",
  );
  const changes = gitText(root, ["diff", "--name-only", "HEAD"]).split("\n").filter(Boolean);
  assert.deepEqual(
    changes,
    role === "before" ? [installerPath] : [],
    "only the declared recipe overlay",
  );
  const expected = role === "before" ? historicalInstaller : authority.installer;
  const recipe = recipeIdentity(readFileSync(resolve(root, installerPath)), expected.revision);
  assert.deepEqual(recipe, expected, "exact physical installer recipe");
  return { recipe, changes };
}

export function prepareInstallRecipe(role, root, expectedSource) {
  assert.ok(["before", "after"].includes(role), "declared installer role");
  const authority = currentInstallSource(root);
  assert.equal(authority.source.revision, expectedSource, "same actual current source");
  assert.equal(
    gitText(root, ["status", "--porcelain", "--untracked-files=all"]),
    "",
    "fresh source checkout",
  );
  if (role === "before") {
    const original = readFileSync(historicalFile);
    assert.deepEqual(
      recipeIdentity(original, baseline),
      historicalInstaller,
      "immutable historical recipe",
    );
    writeFileSync(resolve(root, installerPath), original);
  }
  return { authority, ...inspectInstallRecipe(role, root, authority) };
}

function githubContext(environment) {
  assert.match(environment.GITHUB_SHA ?? "", /^[0-9a-f]{40}$/, "current GitHub actor SHA");
  assert.ok(
    ["pull_request", "workflow_dispatch"].includes(environment.GITHUB_EVENT_NAME),
    "allowed GitHub event",
  );
  assert.ok(environment.GITHUB_EVENT_PATH, "current GitHub event file");
  const event = JSON.parse(readFileSync(environment.GITHUB_EVENT_PATH, "utf8"));
  const triggerHead = event.pull_request?.head?.sha ?? null;
  if (environment.GITHUB_EVENT_NAME === "pull_request") {
    assert.match(triggerHead ?? "", /^[0-9a-f]{40}$/, "current PR trigger SHA");
  } else assert.equal(triggerHead, null, "manual dispatch has no PR trigger");
  return { githubSha: environment.GITHUB_SHA, event: environment.GITHUB_EVENT_NAME, triggerHead };
}

async function capture(role, directory, destination) {
  assert.ok(["before", "after"].includes(role));
  const context = githubContext(process.env);
  const root = resolve(directory);
  const output = resolve(destination);
  mkdirSync(output, { recursive: true });
  const run = (name, command, args, cwd = root) => {
    const result = spawnSync(command, args, { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
    writeFileSync(resolve(output, `${name}.stdout`), result.stdout ?? "");
    writeFileSync(resolve(output, `${name}.stderr`), result.stderr ?? "");
    if (result.error) throw result.error;
    return {
      command: [command, ...args],
      status: result.status,
      signal: result.signal,
      stdout: result.stdout,
      stderr: result.stderr,
    };
  };
  const actor = run("observer", "git", ["rev-parse", "HEAD"], observer).stdout.trim();
  assert.equal(actor, context.githubSha);
  const source = run("source", "git", ["rev-parse", "HEAD"]).stdout.trim();
  assert.equal(source, actor);
  const prepared = prepareInstallRecipe(role, root, actor);
  writeFileSync(resolve(output, "source-recipe.json"), JSON.stringify(prepared, null, 2) + "\n");
  // The recipe checkout and source checkout are both fresh. No parent install may mask a missing issuer dependency.
  for (let parent = root; ; parent = dirname(parent)) {
    assert.equal(existsSync(resolve(parent, "node_modules")), false, parent);
    if (parent === dirname(parent)) break;
  }
  assert.equal(existsSync(resolve(root, "npm/builder/shared/node_modules")), false);
  const originalInputs = hashes(root);
  const manager = run("package-manager", "vp", ["env", "exec", "pnpm", "--version"]);
  assert.equal(manager.status, 0, manager.stderr);
  assert.equal(manager.stdout.trim(), "12.1.0");
  const old = process.cwd();
  let filters;
  try {
    process.chdir(root);
    filters = (
      await import(pathToFileURL(resolve(root, "tools/support/compat/github/install-js-checks.ts")))
    ).jsInstallFilters("builders");
  } finally {
    process.chdir(old);
  }
  const selection = run("selection", "vp", [
    "list",
    "--",
    ...filters.flatMap((filter) => ["--filter", filter]),
    "--depth",
    "-1",
    "--json",
  ]);
  assert.equal(selection.status, 0, selection.stderr);
  const projects = JSON.parse(selection.stdout)
    .map(({ name }) => name)
    .sort(compareNames);
  assert.equal(projects.includes("@vizejs/builder-shared"), role === "after");
  const started = performance.now();
  const install = run("install", process.execPath, [
    "tools/support/compat/github/install-js-checks.ts",
    "builders",
  ]);
  assert.equal(install.status, 0, install.stderr);
  const installMilliseconds = performance.now() - started;
  const probe = run("issuer", process.execPath, [
    "--input-type=module",
    "-e",
    `
    import { createRequire } from 'node:module';
    import { readFileSync } from 'node:fs';
    import { pathToFileURL } from 'node:url';
    try {
      await import(pathToFileURL(process.cwd() + '/npm/builder/shared/mapped-module.ts'));
      const require = createRequire(pathToFileURL(process.cwd() + '/npm/builder/shared/package.json'));
      const filename = require.resolve('webpack-sources/package.json');
      console.log(JSON.stringify({ filename, version: JSON.parse(readFileSync(filename)).version }));
    } catch (error) {
      console.error(error.stack);
      console.log(JSON.stringify({ code: error.code, message: error.message }));
      process.exitCode = 1;
    }
  `,
  ]);
  const tests = run("source-maps", process.execPath, [
    "--test",
    "npm/builder/rspack/src/shared/source-map.test.ts",
  ]);
  const issuer = JSON.parse(probe.stdout);
  if (role === "before") {
    assert.equal(probe.status, 1);
    assert.equal(issuer.code, "ERR_MODULE_NOT_FOUND");
    assert.match(
      issuer.message,
      /Cannot find package 'webpack-sources' imported from .*npm\/builder\/shared\/mapped-module\.ts/,
    );
    assert.equal(tests.status, 1);
    assert.match(tests.stdout + tests.stderr, /Cannot find package 'webpack-sources'/);
  } else {
    assert.equal(probe.status, 0, probe.stderr);
    assert.equal(issuer.version, "3.5.1");
    assert.equal(tests.status, 0, tests.stdout + tests.stderr);
  }
  assert.deepEqual(hashes(root), originalInputs, "frozen install preserves source and lock bytes");
  assert.deepEqual(
    inspectInstallRecipe(role, root, prepared.authority),
    {
      recipe: prepared.recipe,
      changes: prepared.changes,
    },
    "install preserves the authenticated source and recipe",
  );
  const packet = {
    schema,
    role,
    source: prepared.authority.source,
    recipe: prepared.recipe,
    physicalChanges: prepared.changes,
    observerActor: actor,
    ...context,
    node: process.version,
    packageManager: JSON.parse(readFileSync(resolve(root, "package.json"))).packageManager,
    installedPackageManager: manager.stdout.trim(),
    inputs: originalInputs,
    filters,
    projects,
    install: { status: install.status, milliseconds: installMilliseconds },
    issuer: { ...issuer, status: probe.status },
    tests: { status: tests.status },
    complete: true,
  };
  writeFileSync(resolve(output, "qualification.json"), JSON.stringify(packet, null, 2) + "\n");
  console.log(JSON.stringify(packet, null, 2));
}

export function compareInstallClosure(directory, environment = process.env) {
  const context = githubContext(environment);
  const authority = currentInstallSource();
  assert.equal(authority.source.revision, context.githubSha, "current observer source");
  const packets = ["before", "after"].map((role) => {
    const packet = JSON.parse(readFileSync(resolve(directory, role, "qualification.json")));
    assert.equal(packet.schema, schema, "explicit source/recipe schema");
    assert.equal(packet.role, role);
    assert.equal(packet.complete, true);
    assert.equal(packet.observerActor, context.githubSha);
    for (const key of ["githubSha", "event", "triggerHead"])
      assert.equal(packet[key], context[key], key);
    assert.deepEqual(packet.source, authority.source, "same actual current source");
    assert.deepEqual(packet.inputs, authority.inputs, "actual current source inputs");
    assert.deepEqual(
      packet.recipe,
      role === "before" ? historicalInstaller : authority.installer,
      "authenticated installer recipe",
    );
    assert.deepEqual(
      packet.physicalChanges,
      role === "before" ? [installerPath] : [],
      "only the declared recipe overlay",
    );
    assert.equal(packet.install.status, 0);
    assert.equal(packet.issuer.status, role === "before" ? 1 : 0);
    assert.equal(packet.tests.status, role === "before" ? 1 : 0);
    return packet;
  });
  const [before, after] = packets;
  assert.deepEqual(
    after.inputs,
    before.inputs,
    "same declared dependencies, lock, emitter and real mapping tests",
  );
  assert.deepEqual(
    after.projects,
    [...before.projects, "@vizejs/builder-shared"].sort(compareNames),
  );
  for (const key of ["event", "triggerHead", "node", "packageManager", "installedPackageManager"])
    assert.equal(after[key], before[key], key);
  assert.equal(before.issuer.code, "ERR_MODULE_NOT_FOUND");
  assert.equal(after.issuer.version, "3.5.1");
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  if (process.argv[2] === "compare") {
    compareInstallClosure(resolve(process.argv[3]));
    console.log(
      "Fresh exact-command before/after install closure and unchanged source-map tests qualified.",
    );
  } else await capture(...process.argv.slice(2));
}
