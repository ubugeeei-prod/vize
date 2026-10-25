import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export function resolveCssEventSource({ event, sha, ref, head, literalHead, parents }) {
  assert.match(sha, /^[0-9a-f]{40}$/u);
  assert.equal(head, sha, "actual checkout must equal the event source");
  if (event === "pull_request") {
    assert.match(ref, /^refs\/pull\/[1-9][0-9]*\/merge$/u);
    assert.match(literalHead, /^[0-9a-f]{40}$/u);
    assert.equal(parents.length, 2, "PR proof requires the actual merge candidate parents");
    assert.ok(
      parents.includes(literalHead),
      "literal PR head must be a parent of the measured merge candidate",
    );
    assert.notEqual(
      sha,
      literalHead,
      "merge evidence must never be relabeled literal-head qualification",
    );
    return {
      role: "pull-request-merge",
      source: sha,
      literalPullRequestHead: literalHead,
      parents,
    };
  }
  assert.equal(event, "workflow_dispatch", "only dedicated PR/dispatch source roles are admitted");
  assert.match(ref, /^refs\/(?:heads|tags)\/.+/u);
  assert.equal(literalHead, "");
  return { role: "dispatch-source", source: sha, literalPullRequestHead: null, parents };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const directory = path.resolve("target/css-compile-source-maps");
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, "event-source.json");
  const attempt = {
    schema: "vize.css.source-map.event-source",
    version: 1,
    status: "attempt",
    event: process.env.GITHUB_EVENT_NAME,
    expectedSource: process.env.VIZE_CSS_MAP_SOURCE_SHA,
    literalPullRequestHead: process.env.VIZE_CSS_MAP_PR_HEAD,
  };
  fs.writeFileSync(file, JSON.stringify(attempt, null, 2) + "\n");
  const head = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  const parents = execFileSync("git", ["rev-list", "--parents", "-n", "1", head], {
    encoding: "utf8",
  })
    .trim()
    .split(" ")
    .slice(1);
  assert.equal(process.env.VIZE_CSS_MAP_SOURCE_SHA, process.env.GITHUB_SHA);
  const source = resolveCssEventSource({
    event: process.env.GITHUB_EVENT_NAME,
    sha: process.env.GITHUB_SHA,
    ref: process.env.GITHUB_REF,
    head,
    literalHead: process.env.VIZE_CSS_MAP_PR_HEAD,
    parents,
  });
  fs.writeFileSync(
    file,
    JSON.stringify({ ...attempt, status: "qualified", ...source }, null, 2) + "\n",
  );
}
