import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  comparisonSources,
  sourceProvenance,
  validateSourceProvenance,
} from "../../tools/support/compat/github/corsa-package-refresh/source.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const recipe = "tools/support/compat/github/corsa-package-refresh";
const workflow = fs.readFileSync(
  path.join(root, ".github/workflows/corsa-package-refresh.yml"),
  "utf8",
);
const compareStep =
  / {6}- name: Compare unchanged tests with one physical historical SDK[\s\S]+? {8}run: \|\n((?: {10}.+\n)+)/.exec(
    workflow,
  )?.[1];
assert.ok(compareStep, "the workflow must keep its comparison step");
const commands = compareStep
  .trimEnd()
  .split("\n")
  .map((line) => line.trim());
const actor = "a".repeat(40);
const head = "b".repeat(40);
const environment = (event = "pull_request", source = actor, triggerHead = head) => ({
  GITHUB_EVENT_NAME: event,
  GITHUB_SHA: source,
  VIZE_PACKAGE_REFRESH_TRIGGER_HEAD: triggerHead,
});

void test("the workflow checks out and names evidence for the actual actor, with a separate PR head", () => {
  // These closed policy expressions are the complete checkout/artifact selector;
  // no alternate expression may silently change the source role under test.
  assert.deepEqual(
    [...workflow.matchAll(/^ {10}ref: (.+)$/gm)].map((match) => match[1]),
    ["${{ github.sha }}"],
  );
  assert.deepEqual(commands, [
    `cp -R ${recipe} "$RUNNER_TEMP/corsa-package-refresh-harness"`,
    'vp node "$RUNNER_TEMP/corsa-package-refresh-harness/compare.mjs"',
    'vp node "$RUNNER_TEMP/corsa-package-refresh-harness/fixture.mjs"',
  ]);
  assert.deepEqual(
    [...workflow.matchAll(/^ {10}name: corsa-package-refresh-(.+)$/gm)].map((match) => match[1]),
    ["${{ github.sha }}"],
  );
  assert.deepEqual(
    [...workflow.matchAll(/^ {10}VIZE_PACKAGE_REFRESH_TRIGGER_HEAD: (.+)$/gm)].map(
      (match) => match[1],
    ),
    ["${{ github.event.pull_request.head.sha || '' }}"],
  );
  assert.deepEqual(
    [...workflow.matchAll(/^ {8}run: vp node --test (.+)$/gm)].map((match) => match[1]),
    ["tests/tooling/corsa-package-refresh-bootstrap.test.ts"],
  );
});

void test("an older PR without the recipe bootstraps from the merged actor without changing its head", (t) => {
  const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-corsa-bootstrap-"));
  t.after(() => fs.rmSync(scratch, { recursive: true, force: true }));
  const checkout = path.join(scratch, "repository");
  fs.mkdirSync(checkout);
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: checkout, encoding: "utf8" }).trim();
  git("init", "-b", "main");
  git("config", "user.name", "Bootstrap fixture");
  git("config", "user.email", "bootstrap@example.invalid");
  git("config", "commit.gpgsign", "false");
  const copyRecipe = (name: string) => {
    const temporary = path.join(scratch, name);
    fs.mkdirSync(temporary);
    const result = spawnSync("bash", ["-euo", "pipefail", "-c", commands[0]], {
      cwd: checkout,
      env: { ...process.env, RUNNER_TEMP: temporary },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    return { ...result, destination: path.join(temporary, "corsa-package-refresh-harness") };
  };
  fs.writeFileSync(path.join(checkout, "base.txt"), "controlled bootstrap only\n");
  git("add", ".");
  git("commit", "-m", "test: base");
  git("checkout", "-b", "older-pr");
  fs.writeFileSync(path.join(checkout, "manifest.json"), '{"changed":true}\n');
  git("add", ".");
  git("commit", "-m", "test: older manifest head");
  const triggerHead = git("rev-parse", "HEAD");
  git("checkout", "main");
  fs.cpSync(path.join(root, recipe), path.join(checkout, recipe), { recursive: true });
  git("add", ".");
  git("commit", "-m", "test: current observer recipe");
  git("merge", "--no-ff", "older-pr", "-m", "test: workflow merge actor");
  const observerActor = git("rev-parse", "HEAD");
  assert.notEqual(observerActor, triggerHead);
  git("checkout", "--detach", triggerHead);
  assert.equal(fs.existsSync(path.join(checkout, recipe)), false);
  const oldCopy = copyRecipe("old-runner");
  assert.equal(oldCopy.status, 1);
  assert.equal(fs.existsSync(oldCopy.destination), false);
  git("checkout", "--detach", observerActor);
  const actorCopy = copyRecipe("merge-runner");
  assert.equal(actorCopy.status, 0, actorCopy.stderr);
  const copiedRecipe = actorCopy.destination;
  for (const file of fs.readdirSync(path.join(root, recipe))) {
    assert.deepEqual(
      fs.readFileSync(path.join(copiedRecipe, file)),
      fs.readFileSync(path.join(root, recipe, file)),
    );
  }
  const script = `import { execFileSync } from 'node:child_process';
    import { sourceProvenance } from ${JSON.stringify(pathToFileURL(path.join(copiedRecipe, "source.mjs")).href)};
    const actual = execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim();
    process.stdout.write(JSON.stringify(sourceProvenance(actual)));`;
  const actual = JSON.parse(
    execFileSync(process.execPath, ["--input-type=module", "--eval", script], {
      cwd: checkout,
      encoding: "utf8",
      env: { ...process.env, ...environment("pull_request", observerActor, triggerHead) },
    }),
  );
  assert.deepEqual(actual, {
    event: "pull_request",
    observerActor,
    githubSha: observerActor,
    triggerHead,
  });
  assert.equal(git("rev-parse", "older-pr"), triggerHead);
  assert.equal(git("show", `${triggerHead}:manifest.json`), '{"changed":true}');
});

void test("source provenance rejects stale checkout, missing or malformed context and altered receipts", () => {
  const env = environment();
  const expected = {
    event: "pull_request",
    observerActor: actor,
    githubSha: actor,
    triggerHead: head,
  };
  assert.deepEqual(sourceProvenance(actor, env), expected);
  assert.deepEqual(validateSourceProvenance(expected, actor, env), expected);
  for (const changed of [
    {},
    { ...env, GITHUB_SHA: "" },
    { ...env, GITHUB_SHA: head },
    { ...env, GITHUB_EVENT_NAME: "pull_request_target" },
    { ...env, VIZE_PACKAGE_REFRESH_TRIGGER_HEAD: "" },
    { ...env, VIZE_PACKAGE_REFRESH_TRIGGER_HEAD: "not-a-sha" },
  ])
    assert.throws(() => sourceProvenance(actor, changed));
  assert.throws(() => sourceProvenance(head, env));
  assert.throws(() => sourceProvenance("not-a-sha", env));
  for (const changed of [
    { ...expected, observerActor: head },
    { ...expected, triggerHead: actor },
    { ...expected, githubSha: head },
    { ...expected, event: "workflow_dispatch" },
    { ...expected, source: head },
    { ...expected, triggerHead: undefined },
  ])
    assert.throws(() => validateSourceProvenance(changed, actor, env));
});

void test("manual observers bind their actual branch while historical products keep immutable sources", () => {
  const env = environment("workflow_dispatch", actor, "");
  assert.deepEqual(sourceProvenance(actor, env), {
    event: "workflow_dispatch",
    observerActor: actor,
    githubSha: actor,
    triggerHead: null,
  });
  assert.throws(() => sourceProvenance(actor, { ...env, VIZE_PACKAGE_REFRESH_TRIGGER_HEAD: head }));
  assert.deepEqual(comparisonSources(actor, false), [["fixed", actor]]);
  assert.deepEqual(comparisonSources(actor, true), [
    ["before-policy", "13fa41356d075047a229f580014b637d3bbeb57f"],
    ["runtime-policy", "9469f272b189d0220a9838539f2c878fc5d70edd"],
    ["fixed", actor],
  ]);
  assert.throws(() => comparisonSources(actor, "true"));
  assert.throws(() => comparisonSources("invalid", true));
});
