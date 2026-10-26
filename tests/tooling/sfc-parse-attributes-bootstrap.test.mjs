import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  baseSource,
  sourceProvenance,
  validateProductSource,
  validateProductPacket,
  validateSourceProvenance,
} from "../native/sfc-parse-attributes-source.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const recipe = "tests/native/sfc-parse-attributes-source.mjs";
const workflow = fs.readFileSync(
  path.join(root, ".github/workflows/sfc-parse-attributes.yml"),
  "utf8",
);
const actor = "a".repeat(40);
const head = "b".repeat(40);
const environment = (event = "pull_request", source = actor, triggerHead = head) => ({
  GITHUB_EVENT_NAME: event,
  GITHUB_SHA: source,
  VIZE_SFC_ATTRS_TRIGGER_HEAD: triggerHead,
});

void test("attributes workflow selects the actual actor and preserves the immutable base", () => {
  assert.match(workflow, /^  VIZE_SFC_ATTRS_ACTOR_SHA: \$\{\{ github\.sha \}\}$/m);
  assert.match(
    workflow,
    /^  VIZE_SFC_ATTRS_TRIGGER_HEAD: \$\{\{ github\.event\.pull_request\.head\.sha \|\| '' \}\}$/m,
  );
  assert.deepEqual(
    [...workflow.matchAll(/with: \{ ref: "([^"]+)", persist-credentials: false \}/g)].map(
      (match) => match[1],
    ),
    ["${{ env.VIZE_SFC_ATTRS_ACTOR_SHA }}", "${{ env.VIZE_SFC_ATTRS_ACTOR_SHA }}"],
  );
  assert.match(workflow, new RegExp(`^          ref: ${baseSource}$`, "m"));
  assert.ok(
    workflow.includes(
      `VIZE_SFC_ATTRS_SOURCE_SHA: \${{ matrix.role == 'base' && '${baseSource}' || github.sha }}`,
    ),
  );
  assert.doesNotMatch(workflow, /VIZE_SFC_ATTRS_HEAD_SHA/);
  assert.match(
    workflow,
    /^          node --test tests\/tooling\/sfc-parse-attributes-bootstrap\.test\.mjs$/m,
  );
  assert.match(workflow, new RegExp(`^          node ${recipe.replaceAll(".", "\\.")}$`, "m"));
  for (const line of workflow
    .split("\n")
    .filter((line) => /name: ["']?sfc-parse-attributes-/.test(line)))
    assert.ok(line.includes("${{ env.VIZE_SFC_ATTRS_ACTOR_SHA }}"), line);
});

void test("an older head missing the real recipe runs it from the merge actor without rewriting the head", (t) => {
  const checkout = fs.mkdtempSync(path.join(os.tmpdir(), "vize-attrs-bootstrap-"));
  t.after(() => fs.rmSync(checkout, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", args, { cwd: checkout, encoding: "utf8" }).trim();
  git("init", "-b", "main");
  git("config", "user.name", "Bootstrap fixture");
  git("config", "user.email", "bootstrap@example.invalid");
  git("config", "commit.gpgsign", "false");
  fs.writeFileSync(path.join(checkout, "base.txt"), "controlled bootstrap only\n");
  git("add", ".");
  git("commit", "-m", "test: base");
  git("checkout", "-b", "older-pr");
  fs.writeFileSync(path.join(checkout, "product.json"), '{"changed":true}\n');
  git("add", ".");
  git("commit", "-m", "test: older product head");
  const triggerHead = git("rev-parse", "HEAD");
  git("checkout", "main");
  fs.mkdirSync(path.dirname(path.join(checkout, recipe)), { recursive: true });
  fs.copyFileSync(path.join(root, recipe), path.join(checkout, recipe));
  git("add", ".");
  git("commit", "-m", "test: current observer recipe");
  git("merge", "--no-ff", "older-pr", "-m", "test: workflow merge actor");
  const observerActor = git("rev-parse", "HEAD");
  assert.notEqual(observerActor, triggerHead);
  const run = () =>
    spawnSync(process.execPath, [recipe], {
      cwd: checkout,
      env: { ...process.env, ...environment("pull_request", observerActor, triggerHead) },
      encoding: "utf8",
      timeout: 10_000,
    });
  git("checkout", "--detach", triggerHead);
  assert.equal(fs.existsSync(path.join(checkout, recipe)), false);
  const missing = run();
  assert.equal(missing.status, 1);
  assert.match(missing.stderr, /MODULE_NOT_FOUND/);
  git("checkout", "--detach", observerActor);
  const actual = run();
  assert.equal(actual.status, 0, actual.stderr);
  assert.deepEqual(JSON.parse(actual.stdout), {
    event: "pull_request",
    observerActor,
    githubSha: observerActor,
    triggerHead,
  });
  assert.deepEqual(
    fs.readFileSync(path.join(checkout, recipe)),
    fs.readFileSync(path.join(root, recipe)),
  );
  assert.equal(git("rev-parse", "older-pr"), triggerHead);
  assert.equal(git("show", `${triggerHead}:product.json`), '{"changed":true}');
});

void test("source roles reject stale actors, malformed triggers and altered producer receipts", () => {
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
    { ...env, VIZE_SFC_ATTRS_TRIGGER_HEAD: "" },
    { ...env, VIZE_SFC_ATTRS_TRIGGER_HEAD: "not-a-sha" },
  ])
    assert.throws(() => sourceProvenance(actor, changed));
  assert.throws(() => sourceProvenance(head, env));
  assert.throws(() => sourceProvenance("not-a-sha", env));
  for (const changed of [
    { ...expected, observerActor: head },
    { ...expected, githubSha: head },
    { ...expected, triggerHead: actor },
    { ...expected, event: "workflow_dispatch" },
    { ...expected, source: head },
    { ...expected, triggerHead: undefined },
  ])
    assert.throws(() => validateSourceProvenance(changed, actor, env));
});

void test("manual observers have no PR trigger while before/head products retain their actual sources", () => {
  const env = environment("workflow_dispatch", actor, "");
  assert.deepEqual(sourceProvenance(actor, env), {
    event: "workflow_dispatch",
    observerActor: actor,
    githubSha: actor,
    triggerHead: null,
  });
  assert.throws(() => sourceProvenance(actor, { ...env, VIZE_SFC_ATTRS_TRIGGER_HEAD: head }));
  assert.equal(baseSource, "49a98d1e72707083354b8ff11e1cec800ded28b3");
  validateProductSource(baseSource, "base", actor);
  validateProductSource(actor, "head", actor);
  assert.throws(() => validateProductSource(head, "head", actor));
  assert.throws(() => validateProductSource(actor, "base", actor));
  assert.throws(() => validateProductSource(baseSource, "other", actor));
  assert.throws(() => validateProductSource(actor, "head", "invalid"));
});

void test("packet, receipt and image roles must all match the independent artifact container", () => {
  const env = environment();
  const receipt = { role: "base", source: baseSource, provenance: sourceProvenance(actor, env) };
  const packet = { role: "base", source: baseSource, receipt };
  validateProductPacket(packet, structuredClone(receipt), "base", actor, env);
  const wrongReceipt = structuredClone(packet);
  wrongReceipt.receipt.role = "head";
  // Matching false labels in both files must not legitimize a base image as head.
  assert.throws(() =>
    validateProductPacket(wrongReceipt, wrongReceipt.receipt, "base", actor, env),
  );
  const wrongPacket = { ...packet, role: "head" };
  assert.throws(() => validateProductPacket(wrongPacket, receipt, "base", actor, env));
  assert.throws(() =>
    validateProductPacket(packet, { ...receipt, role: "head" }, "base", actor, env),
  );
  assert.throws(() => validateProductPacket(packet, receipt, "head", actor, env));
});
