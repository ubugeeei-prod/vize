import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  generateReleaseNotes,
  previousPublishedTag,
  type PublishedRelease,
} from "../../../tools/support/release/github_release_notes.ts";

const release = (
  tag_name: string,
  overrides: Partial<PublishedRelease> = {},
): PublishedRelease => ({
  tag_name,
  draft: false,
  prerelease: tag_name.includes("-"),
  published_at: "2026-10-17T00:00:00Z",
  ...overrides,
});

// Real pinned topology: release metadata commits are siblings of main's later
// integration, not ancestors of the following release's immutable source cut.
test("sibling pinned tags use the preceding published version, not the common ancestor's release", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-release-notes-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  try {
    git("init", "--quiet", "--initial-branch=main");
    git("config", "user.name", "Release Fixture");
    git("config", "user.email", "release@example.invalid");
    const commit = (message: string) => {
      writeFileSync(join(root, "source.txt"), message);
      git("add", "source.txt");
      git("commit", "--quiet", "-m", message);
    };
    commit("old release");
    git("tag", "v0.435.0");
    commit("previous source cut");
    const previousCut = git("rev-parse", "HEAD");
    git("checkout", "--quiet", "-b", "previous-release");
    commit("previous version metadata");
    git("tag", "v0.457.0");
    git("checkout", "--quiet", "main");
    commit("version integration and product fixes");
    const currentCut = git("rev-parse", "HEAD");
    git("checkout", "--quiet", "-b", "current-release");
    commit("current version metadata");
    const currentHead = git("rev-parse", "HEAD");
    git("tag", "v0.458.0");
    assert.equal(git("merge-base", "v0.457.0", "v0.458.0"), previousCut);
    assert.equal(git("rev-parse", "HEAD^"), currentCut);
    assert.equal(
      spawnSync("git", ["merge-base", "--is-ancestor", "v0.457.0", currentHead], { cwd: root })
        .status,
      1,
    );
    const requests: { path: string; body?: object }[] = [];
    const notes = generateReleaseNotes(
      (path, body) => {
        requests.push({ path, body });
        return body
          ? { body: "## What's Changed\n\nOnly current fixes.\n" }
          : [release("v0.458.0"), release("v0.435.0"), release("v0.457.0")];
      },
      "owner/repo",
      "v0.458.0",
      currentHead,
    );
    assert.deepEqual(requests, [
      { path: "repos/owner/repo/releases?per_page=100&page=1", body: undefined },
      {
        path: "repos/owner/repo/releases/generate-notes",
        body: {
          tag_name: "v0.458.0",
          target_commitish: currentHead,
          previous_tag_name: "v0.457.0",
        },
      },
    ]);
    assert.deepEqual(notes, {
      body: "## What's Changed\n\nOnly current fixes.\n",
      previousTag: "v0.457.0",
    });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("publication timestamps, newer releases, drafts, and prereleases cannot widen a stable range", () => {
  assert.equal(
    previousPublishedTag("v0.458.0", [
      release("v0.459.0"),
      release("v0.458.0"),
      release("v0.457.1", { draft: true }),
      release("v0.458.0-rc.10"),
      release("v0.456.0", { published_at: "2027-01-01T00:00:00Z" }),
      release("v0.457.0"),
      release("standalone-assets"),
      release("v0.457.2", { published_at: null }),
    ]),
    "v0.457.0",
  );
  assert.equal(
    previousPublishedTag("v0.458.0-rc.11", [
      release("v0.458.0-rc.2"),
      release("v0.458.0-rc.10"),
      release("v0.458.0-rc.12"),
      release("v0.458.0"),
    ]),
    "v0.458.0-rc.10",
  );
  assert.equal(
    previousPublishedTag("v0.458.0", [release("v0.99.0"), release("v0.457.0")]),
    "v0.457.0",
  );
  assert.throws(() => previousPublishedTag("v01.2.3", []), /Invalid release version/);
  assert.throws(() => previousPublishedTag("v1.2.3-rc.01", []), /Invalid numeric prerelease/);
});

test("pagination finds a predecessor beyond 100 entries and missing published baselines fail before generation", () => {
  const requests: object[] = [];
  const result = generateReleaseNotes(
    (path, body) => {
      if (body) {
        requests.push(body);
        return { body: "bounded notes" };
      }
      return path.endsWith("page=1")
        ? Array.from({ length: 100 }, (_, index) => release(`v0.${1000 + index}.0`))
        : [release("v0.457.0")];
    },
    "owner/repo",
    "v0.458.0",
    "a".repeat(40),
  );
  assert.equal(result.previousTag, "v0.457.0");
  assert.deepEqual(requests, [
    { tag_name: "v0.458.0", target_commitish: "a".repeat(40), previous_tag_name: "v0.457.0" },
  ]);
  for (const candidates of [
    [],
    [release("v0.1.0")],
    [release("v0.1.0-rc.1")],
    [release("v0.0.9", { draft: true })],
  ]) {
    const called: object[] = [];
    assert.throws(
      () =>
        generateReleaseNotes(
          (_path, body) => {
            if (body) called.push(body);
            return candidates;
          },
          "owner/repo",
          "v0.1.0",
          "b".repeat(40),
        ),
      /No preceding published version/,
    );
    assert.deepEqual(called, [], "never ask GitHub to infer an unpublished or prerelease baseline");
  }
  assert.throws(
    () =>
      generateReleaseNotes(() => [release("v0.459.0")], "owner/repo", "v0.458.0", "a".repeat(40)),
    /No preceding/,
  );
  assert.throws(
    () => generateReleaseNotes(() => [], "owner/repo", "v0.458.0", "main"),
    /exact source SHA/,
  );
});

test("the workflow publishes only the prepared body at the pinned source and removes historical static notes", () => {
  const workflow = readFileSync(
    new URL("../../../.github/workflows/release.yml", import.meta.url),
    "utf8",
  );
  const job = workflow.split("  create-github-release:")[1].split("  release-npm-cli:")[0];
  assert.ok(
    job.includes(
      'node tools/support/release/github_release_notes.ts "$RELEASE_TAG_NAME" "$RELEASE_SOURCE_SHA" "$RUNNER_TEMP/release-notes.md"',
    ),
  );
  assert.ok(job.includes("RELEASE_SOURCE_SHA: ${{ inputs.expected_sha }}"));
  assert.ok(job.includes("body_path: ${{ runner.temp }}/release-notes.md"));
  assert.ok(job.includes("generate_release_notes: false"));
  assert.ok(job.includes("target_commitish: ${{ inputs.expected_sha }}"));
  assert.ok(
    job.indexOf("actions/setup-node@") <
      job.indexOf("node tools/support/release/github_release_notes.ts"),
  );
  assert.ok(
    job.indexOf("Generate notes for the exact published version range") <
      job.indexOf("name: Create Release"),
  );
  const scope = readFileSync(
    new URL("../../../.github/release-notes/drop-in-scope.md", import.meta.url),
    "utf8",
  );
  assert.doesNotMatch(scope, /0\.\d+\.\d+|^## (?!Drop-in scope$)/m);
});

test("the real command preserves generated notes and evergreen scope and refuses a different checkout", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-release-notes-command-"));
  const script = new URL("../../../tools/support/release/github_release_notes.ts", import.meta.url);
  try {
    const git = (...args: string[]) =>
      execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
    git("init", "--quiet");
    git(
      "-c",
      "user.name=Release Fixture",
      "-c",
      "user.email=release@example.invalid",
      "commit",
      "--allow-empty",
      "--quiet",
      "-m",
      "immutable source",
    );
    const sha = git("rev-parse", "HEAD");
    const generated =
      "Large complete generated body " +
      "x".repeat(2 * 1024 * 1024) +
      "\n## What's Changed\n\n* Actual current fix by @reporter in https://github.com/owner/repo/pull/1\n\n**Full Changelog**: https://github.com/owner/repo/compare/v0.457.0...v0.458.0\n";
    const gh = join(root, "gh");
    writeFileSync(
      gh,
      `#!${process.execPath}\nconst fs = require('node:fs');\nconst body = fs.readFileSync(0, 'utf8');\nfs.appendFileSync(${JSON.stringify(join(root, "api.jsonl"))}, JSON.stringify({args:process.argv.slice(2),body})+'\\n');\nprocess.stdout.write(JSON.stringify(body ? {body:${JSON.stringify(generated)}} : ${JSON.stringify([release("v0.458.0"), release("v0.457.0")])}));\n`,
      { mode: 0o755 },
    );
    const output = join(root, "notes.md");
    const env = {
      ...process.env,
      GITHUB_REPOSITORY: "owner/repo",
      PATH: `${root}:${process.env.PATH ?? ""}`,
    };
    const result = spawnSync(process.execPath, [script.pathname, "v0.458.0", sha, output], {
      cwd: root,
      env,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr);
    const scope = readFileSync(
      new URL("../../../.github/release-notes/drop-in-scope.md", import.meta.url),
      "utf8",
    );
    assert.equal(readFileSync(output, "utf8"), `${scope.trim()}\n\n${generated.trim()}\n`);
    const requests = readFileSync(join(root, "api.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.deepEqual(requests, [
      {
        args: [
          "api",
          "repos/owner/repo/releases?per_page=100&page=1",
          "--jq",
          "map({tag_name, draft, prerelease, published_at})",
        ],
        body: "",
      },
      {
        args: [
          "api",
          "repos/owner/repo/releases/generate-notes",
          "--method",
          "POST",
          "--input",
          "-",
        ],
        body: JSON.stringify({
          tag_name: "v0.458.0",
          target_commitish: sha,
          previous_tag_name: "v0.457.0",
        }),
      },
    ]);
    const refused = spawnSync(
      process.execPath,
      [script.pathname, "v0.458.0", "c".repeat(40), output],
      { cwd: root, env, encoding: "utf8" },
    );
    assert.equal(refused.status, 1);
    assert.match(refused.stderr, /checkout differs from the pinned source/);
    assert.equal(
      readFileSync(join(root, "api.jsonl"), "utf8").trim().split("\n").length,
      2,
      "mismatched source must not invoke GitHub",
    );
    assert.equal(readFileSync(output, "utf8"), `${scope.trim()}\n\n${generated.trim()}\n`);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
