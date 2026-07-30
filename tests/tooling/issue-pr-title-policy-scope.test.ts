import assert from "node:assert/strict";
import { test } from "node:test";

import { assignCall, patchTitleCall, runPolicy } from "./support/title-policy.ts";

const prNumber = 101;
const issueNumber = 202;

function prPayload(title: string, action = "edited") {
  return {
    action,
    pull_request: { number: prNumber, title, assignees: [{ login: "ubugeeei" }] },
  };
}

function issuePayload(
  title: string,
  action = "edited",
  assignees: unknown[] = [{ login: "ubugeeei" }],
) {
  return { action, issue: { number: issueNumber, title, assignees } };
}

function runPr(title: string, action?: string) {
  return runPolicy(prPayload(title, action), "pull_request_target");
}

// Every mapping in `title_replacement`, exercised in scope position. This is the
// only position the tool is allowed to rewrite.
const scopeRewrites: ReadonlyArray<readonly [string, string]> = [
  ["fix(check): skip type-check on hidden files", "fix(canon): skip type-check on hidden files"],
  ["fix(compiler): recover Vue compiler quirks", "fix(atelier): recover Vue compiler quirks"],
  ["perf(lint): stream quiet lint aggregation", "perf(patina): stream quiet lint aggregation"],
  ["perf(linter): stream quiet lint aggregation", "perf(patina): stream quiet lint aggregation"],
  ["test(story): add a story for the button", "test(musea): add a story for the button"],
  ["fix(format): format the output", "fix(glyph): format the output"],
  ["fix(fmt): keep fmt idempotent", "fix(glyph): keep fmt idempotent"],
  // Multi-segment scope: `/` is the only segment separator (see normalize_scope).
  ["fix(check/lint): share the resolver", "fix(canon/patina): share the resolver"],
  ["fix(check/cli/fmt): share the resolver", "fix(canon/cli/glyph): share the resolver"],
  // The breaking-change marker sits outside the scope and must survive.
  ["feat(check)!: drop the legacy flag", "feat(canon)!: drop the legacy flag"],
];

test("rewrites area names in the conventional scope and nowhere else", () => {
  for (const [title, expected] of scopeRewrites) {
    const { result, ghCalls } = runPr(title);
    assert.equal(result.status, 0, `${title}\n${result.stderr}`);
    assert.deepEqual(ghCalls, [patchTitleCall(prNumber, expected)], title);
  }
});

// Titles whose subject prose (or scope) merely *mentions* an area word. None of
// these may produce an API call at all: the title is already correct English.
const untouchedPrTitles = [
  // The live regression: PR #3394 was renamed to "rank type-canon engine classes".
  "bench(tools): rank type-check engine classes separately",
  // Landed on main as "fix(cli): canon sources under ..." — not English.
  "fix(cli): check sources under an explicitly included hidden directory",
  // Landed on main as "vize canon benchmark gate" — mangled a real command name.
  "ci(bench): fail-closed vize check benchmark gate with reproducibility metadata",
  "perf(cli): stream quiet lint aggregation",
  "fix(parser): recover Vue compiler quirks",
  "test(sfc): add Vue Router compiler oracle",
  "fix(cli): preserve failed compiler artifacts",
  // No scope at all: nothing is eligible for rewriting.
  "docs: format the output",
  "chore: check the story fixtures",
  // Area name as a substring of a longer scope: must not become `canoner`.
  "fix(checker): tighten diagnostics",
  "fix(formatter): tighten diagnostics",
  // `-`, `.` and `_` are scope characters but not segment separators, so a
  // compound scope is matched whole and therefore left alone.
  "fix(type-check): tighten diagnostics",
  "fix(check.js): tighten diagnostics",
  "fix(check_js): tighten diagnostics",
];

test("leaves subject prose and non-matching scopes untouched", () => {
  for (const title of untouchedPrTitles) {
    const { result, ghCalls } = runPr(title);
    assert.equal(result.status, 0, `${title}\n${result.stderr}`);
    assert.deepEqual(ghCalls, [], title);
  }
});

test("an uppercase scope is not a conventional scope, so it is rejected unchanged", () => {
  // `is_scope_char` is lowercase-only, so `fix(Check): ...` has no conventional
  // prefix: nothing is rewritten and the PR gate fails. Pinned deliberately —
  // silently lower-casing a title the author typed is the same class of bug.
  const { result, ghCalls } = runPr("fix(Check): resolve alias imports");
  assert.equal(result.status, 1);
  assert.deepEqual(ghCalls, []);
});

test("issue titles are normalized by scope only, never in prose", () => {
  const { result, ghCalls } = runPolicy(
    issuePayload("check compiler lint linter story format fmt checklist formatting"),
    "issues",
  );
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, []);
});

test("issue titles with a conventional scope are still normalized", () => {
  const { result, ghCalls } = runPolicy(
    issuePayload("fix(fmt): formatting drifts on every second run", "opened", []),
    "issues",
  );
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(ghCalls, [
    patchTitleCall(issueNumber, "fix(glyph): formatting drifts on every second run"),
    assignCall(issueNumber),
  ]);
});

test("a non-conventional PR title still fails without any rewrite", () => {
  const { result, ghCalls } = runPr("fix lint issue");
  assert.equal(result.status, 1);
  assert.equal(
    result.stdout,
    "::error title=Invalid PR title::Use Conventional Commits format: type(scope): summary\n",
  );
  assert.deepEqual(ghCalls, []);
});
