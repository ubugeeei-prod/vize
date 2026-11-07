import assert from "node:assert/strict";
import { test } from "node:test";

import {
  classifyCommit,
  parseChangelogArgs,
  renderChangelog,
} from "../../../tools/commands/release/regenerate-changelog.ts";

test("changelog arguments are local and fail closed", () => {
  assert.deepEqual(parseChangelogArgs([]), {
    check: false,
    latest: false,
    output: "CHANGELOG.md",
    stdout: false,
    tag: undefined,
    unreleased: false,
  });
  assert.deepEqual(parseChangelogArgs(["--unreleased", "--latest", "--tag", "v1.2.3"]), {
    check: false,
    latest: true,
    output: "CHANGELOG.md",
    stdout: false,
    tag: "v1.2.3",
    unreleased: true,
  });
  assert.throws(() => parseChangelogArgs(["--tag"]), /Missing value for --tag/);
  assert.throws(() => parseChangelogArgs(["--tag", "next"]), /Invalid release tag/);
  assert.throws(() => parseChangelogArgs(["--bogus"]), /Unknown argument/);
  assert.throws(() => parseChangelogArgs(["--check", "--stdout"]), /cannot be combined/);
});

test("conventional commits use the repository release sections", () => {
  assert.equal(classifyCommit("a".repeat(40), "chore: internal cleanup"), undefined);
  assert.equal(classifyCommit("a".repeat(40), "not conventional"), undefined);
  assert.deepEqual(classifyCommit("a".repeat(40), "feat(cli): add JSON output"), {
    id: "a".repeat(40),
    message: "Add JSON output",
    section: "Added",
  });
  assert.equal(classifyCommit("b".repeat(40), "fix!: remove old option")?.section, "Breaking");
  assert.equal(
    classifyCommit("c".repeat(40), "fix(parser): preserve ranges", "BREAKING CHANGE: range DTO")
      ?.section,
    "Breaking",
  );
  assert.equal(
    classifyCommit("d".repeat(40), "docs(release): explain provenance")?.section,
    "Documented release contract",
  );
});

test("changelog rendering is stable and grouped", () => {
  const markdown = renderChangelog([
    {
      version: "v1.2.3",
      date: "2026-11-07",
      commits: [
        { id: "a".repeat(40), message: "Repair parser", section: "Fixed" },
        { id: "b".repeat(40), message: "Add CLI mode", section: "Added" },
      ],
    },
  ]);
  assert.match(markdown, /^# Changelog/m);
  assert.match(markdown, /^## \[1\.2\.3\] - 2026-11-07$/m);
  assert.ok(markdown.indexOf("### Added") < markdown.indexOf("### Fixed"));
  assert.match(markdown, /\/commit\/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/);
  assert.ok(markdown.endsWith("\n"));
});
