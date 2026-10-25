import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const baseline = "fc745ea5a2c9efcf6b1549d6312c61d4f2770697";
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
  assert.equal(source, role === "before" ? baseline : actor);
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
  const packet = {
    role,
    source,
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
  const packets = ["before", "after"].map((role) => {
    const packet = JSON.parse(readFileSync(resolve(directory, role, "qualification.json")));
    assert.equal(packet.role, role);
    assert.equal(packet.complete, true);
    assert.equal(packet.observerActor, context.githubSha);
    for (const key of ["githubSha", "event", "triggerHead"])
      assert.equal(packet[key], context[key], key);
    assert.equal(packet.source, role === "before" ? baseline : context.githubSha);
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
