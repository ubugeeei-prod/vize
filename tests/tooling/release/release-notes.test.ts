import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { documentationReleaseNote } from "../../../docs/scripts/generate-contract-metadata.ts";
import {
  coAuthorCredits,
  generateReleaseNotes,
  previousPublishedTag,
} from "../../../tools/support/release/github_release_notes.ts";
import {
  coAuthorFixtures,
  incompleteMetadataResponses,
  integrationLineEndingCases,
  metadataPage,
  unrecognisedIntegrationBodies,
  release,
  publishedIntegrationIdentity,
  publishedIntegrationFixture,
  unprovedIntegrationMutations,
  writeFakeGh,
} from "./release-notes-fixtures.ts";

type Job = {
  env?: Record<string, string>;
  steps?: Array<{ name?: string; uses?: string; run?: string; with?: Record<string, unknown> }>;
};

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
        return path === "graphql"
          ? metadataPage([release("v0.458.0"), release("v0.435.0"), release("v0.457.0")])
          : { body: "## What's Changed\n\nOnly current fixes.\n" };
      },
      "owner/repo",
      "v0.458.0",
      currentHead,
    );
    const query = (requests[0].body as { query: string }).query;
    assert.match(query, /nodes \{ tagName isDraft isPrerelease publishedAt \}/u);
    assert.match(query, /releases\(first:100,after:\$cursor,/u);
    assert.doesNotMatch(query, /\b(?:assets|description|body)\b/u);
    assert.deepEqual(requests, [
      {
        path: "graphql",
        body: { query, variables: { owner: "owner", name: "repo", cursor: null } },
      },
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
      if (path !== "graphql") {
        assert.ok(body);
        requests.push(body);
        return { body: "bounded notes" };
      }
      const cursor = (body as { variables: { cursor: string | null } }).variables.cursor;
      return cursor === null
        ? metadataPage(
            Array.from({ length: 100 }, (_, index) => release(`v0.${1000 + index}.0`)),
            true,
            "page-100",
          )
        : metadataPage([release("v0.457.0")]);
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
          (path, body) => {
            if (path !== "graphql") {
              assert.ok(body);
              called.push(body);
            }
            return metadataPage(candidates);
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
      generateReleaseNotes(
        () => metadataPage([release("v0.459.0")]),
        "owner/repo",
        "v0.458.0",
        "a".repeat(40),
      ),
    /No preceding/,
  );
  assert.throws(
    () => generateReleaseNotes(() => [], "owner/repo", "v0.458.0", "main"),
    /exact source SHA/,
  );
});

test("release pagination follows pageInfo even for short intermediate and full final pages", () => {
  const cursors: unknown[] = [];
  const generated: object[] = [];
  const result = generateReleaseNotes(
    (path, body) => {
      if (path !== "graphql") {
        assert.ok(body);
        generated.push(body);
        return { body: "complete range" };
      }
      const cursor = (body as { variables: { cursor: string | null } }).variables.cursor;
      cursors.push(cursor);
      return cursor === null
        ? metadataPage([release("v9.0.0")], true, "last-page")
        : metadataPage([
            ...Array.from({ length: 99 }, (_, i) => release(`standalone-${i}`)),
            release("v0.457.0"),
          ]);
    },
    "owner/repo",
    "v0.458.0",
    "a".repeat(40),
  );
  assert.deepEqual(cursors, [null, "last-page"]);
  assert.equal(result.previousTag, "v0.457.0");
  assert.deepEqual(generated, [
    { tag_name: "v0.458.0", target_commitish: "a".repeat(40), previous_tag_name: "v0.457.0" },
  ]);
});

test("incomplete GraphQL data and repeated cursors fail before release-note generation", () => {
  for (const response of incompleteMetadataResponses()) {
    const paths: string[] = [];
    assert.throws(
      () =>
        generateReleaseNotes(
          (path) => {
            paths.push(path);
            return response;
          },
          "owner/repo",
          "v0.458.0",
          "a".repeat(40),
        ),
      /GitHub release/,
    );
    assert.deepEqual(
      paths,
      ["graphql"],
      "never generate a body from incomplete publication history",
    );
  }
  const pages = ["first", "second", "first"].map((cursor) =>
    metadataPage([release("v0.457.0")], true, cursor),
  );
  const paths: string[] = [];
  assert.throws(
    () =>
      generateReleaseNotes(
        (path) => {
          paths.push(path);
          return pages.shift();
        },
        "owner/repo",
        "v0.458.0",
        "a".repeat(40),
      ),
    /pagination did not advance/,
  );
  assert.deepEqual(paths, ["graphql", "graphql", "graphql"]);
});

test("the workflow publishes only the prepared body for the release tag commit", () => {
  const workflow = parse(
    readFileSync(new URL("../../../.github/workflows/release.yml", import.meta.url), "utf8"),
  ) as { jobs: Record<string, Job> };
  const job = workflow.jobs["github-release"];
  const steps = job.steps ?? [];
  const node = steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@"));
  const notes = steps.findIndex((step) => step.name === "Generate release notes");
  const create = steps.findIndex((step) => step.name === "Create Release");
  assert.equal(
    steps[notes]?.run,
    'node tools/support/release/github_release_notes.ts "$RELEASE_TAG" "$RELEASE_SHA" "$RUNNER_TEMP/release-notes.md"',
  );
  assert.equal(job.env?.RELEASE_SHA, "${{ needs.version.outputs.sha }}");
  assert.equal(steps[create]?.with?.body_path, "${{ runner.temp }}/release-notes.md");
  assert.equal(steps[create]?.with?.generate_release_notes, false);
  assert.ok(node >= 0 && node < notes && notes < create);
  const preflight = readFileSync(
    new URL("../../../.github/workflows/release-contracts.yml", import.meta.url),
    "utf8",
  );
  assert.ok(preflight.includes("          tests/tooling/release/release-notes.test.ts\n"));
  assert.ok(preflight.includes("          tests/tooling/drop-in-claim-scope.test.ts\n"));
  for (const input of [
    ".github/workflows/release*.yml",
    ".github/release-notes/drop-in-scope.md",
    "tests/tooling/drop-in-claim-scope.test.ts",
  ]) {
    assert.ok(preflight.includes(`      - "${input}"\n`));
  }
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
    const commits = {
      commits: [
        {
          commit: {
            message:
              "fix: x (#1)\n\nCo-authored-by: reporter <7+reporter@users.noreply.github.com>",
          },
          author: { login: "maintainer" },
        },
      ],
    };
    writeFakeGh(root, generated, commits);
    const thanks = "## Thanks\n\nReported and co-authored by @reporter (#1).\n";
    const output = join(root, "notes.md");
    const env = {
      ...process.env,
      CRATES_PUBLICATION: "publish",
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
    assert.equal(
      readFileSync(output, "utf8"),
      `${scope.trim()}\n\n${generated.trim()}\n\n${documentationReleaseNote()}\n\n${thanks}`,
    );
    const requests = readFileSync(join(root, "api.jsonl"), "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    const query = JSON.parse(requests[0].body).query;
    assert.match(query, /nodes \{ tagName isDraft isPrerelease publishedAt \}/u);
    assert.deepEqual(requests, [
      {
        args: ["api", "graphql", "--method", "POST", "--input", "-"],
        body: JSON.stringify({ query, variables: { owner: "owner", name: "repo", cursor: null } }),
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
      { args: ["api", "repos/owner/repo/git/ref/tags/v0.457.0"], body: "" },
      {
        args: [
          "api",
          `repos/owner/repo/compare/v0.457.0...${sha}?per_page=100&page=1`,
          "--jq",
          "{commits: [.commits[] | {commit: {message: .commit.message}, author}]}",
        ],
        body: "",
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
      4,
      "mismatched source must not invoke GitHub",
    );
    assert.equal(
      readFileSync(output, "utf8"),
      `${scope.trim()}\n\n${generated.trim()}\n\n${documentationReleaseNote()}\n\n${thanks}`,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

void test("only the authenticated already-published integration bullet is removed byte-for-byte", () => {
  const { repository, tag, object, head, merge } = publishedIntegrationIdentity;
  const url = `https://github.com/${repository}/pull/73`;
  const row = `* Any title, not a classification by @reporter in ${url}`;
  const original = `## What's Changed\n* Real fix by @user in https://github.com/${repository}/pull/74\n${row}\n\n**Full Changelog**: https://github.com/${repository}/compare/${tag}...v0.458.0\n`;
  const expected = original.replace(`${row}\n`, "");
  const make = publishedIntegrationFixture;
  const run = (fixture = make(), body = original) => {
    const calls: string[] = [];
    const objects: Record<string, unknown> = {
      [`git/ref/tags/${tag}`]: fixture.reference,
      [`git/tags/${object}`]: fixture.annotated,
      "pulls/73": fixture.pr,
      [`commits/${head}`]: fixture.h,
      [`commits/${merge}`]: fixture.m,
    };
    const notes = generateReleaseNotes(
      (path, request) => {
        calls.push(path);
        if (path === "graphql") return metadataPage([release(tag)]);
        if (request) return { body };
        const key = path.slice(`repos/${repository}/`.length);
        if (!(key in objects)) throw new Error("unavailable identity");
        return objects[key];
      },
      repository,
      "v0.458.0",
      "1".repeat(40),
    );
    return { notes, calls };
  };
  const positive = run();
  assert.deepEqual(positive.notes, { body: expected, previousTag: tag });
  assert.deepEqual(positive.calls, [
    "graphql",
    `repos/${repository}/releases/generate-notes`,
    ...[
      `git/ref/tags/${tag}`,
      `git/tags/${object}`,
      "pulls/73",
      `commits/${head}`,
      `commits/${merge}`,
    ].map((path) => `repos/${repository}/${path}`),
  ]);
  for (const mutate of unprovedIntegrationMutations) {
    const fixture = make();
    mutate(fixture);
    assert.equal(run(fixture).notes.body, original, `unproved identity: ${String(mutate)}`);
  }
  for (const body of unrecognisedIntegrationBodies(original, row, url))
    assert.equal(run(make(), body).notes.body, body, "unrecognized/ambiguous rows stay whole");
  for (const [body, removed] of integrationLineEndingCases(original, row)) {
    assert.equal(run(make(), body).notes.body, removed ? body.replace(removed, "") : body);
  }
  const titledRow = row.replace(
    "Any title, not a classification",
    "chore(release): genuine product content",
  );
  const titledBody = original.replace(row, titledRow);
  assert.equal(run(make(), titledBody).notes.body, titledBody.replace(`${titledRow}\n`, ""));
});

test("release notes credit third-party co-authors with their PRs", () => {
  const fixtures = coAuthorFixtures();
  assert.equal(
    coAuthorCredits(fixtures.commits),
    "## Thanks\n\nReported and co-authored by @escaton (#9768), @naitokosuke (#9759, #9770), Jane Doe (#3).\n",
  );
  assert.equal(coAuthorCredits(fixtures.botOnly), "");
});
