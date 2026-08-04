import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile } from "./support/github-workflows.ts";

const ADVISORY_ENTRY = /^\s*"(RUSTSEC-\d{4}-\d{4})"\s*,?\s*$/;

type IgnoredAdvisory = { id: string; justification: string[] };

/**
 * Collect each ignored advisory together with the comment block directly above
 * it. Any non-comment line resets the block, so a justification cannot be
 * inherited from an unrelated entry.
 */
function ignoredAdvisories(config: string): IgnoredAdvisory[] {
  const entries: IgnoredAdvisory[] = [];
  let comments: string[] = [];
  for (const line of config.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("#")) {
      comments.push(trimmed.slice(1).trim());
      continue;
    }
    const match = ADVISORY_ENTRY.exec(line);
    if (match) {
      entries.push({ id: match[1], justification: comments });
    }
    comments = [];
  }
  return entries;
}

test("cargo-audit ignore entries parse only from the advisory list", () => {
  const entries = ignoredAdvisories(
    [
      "# why: reason",
      "# tracking: #1",
      '  "RUSTSEC-2020-0001",',
      "",
      '  "RUSTSEC-2020-0002",',
    ].join("\n"),
  );

  assert.deepEqual(
    entries.map((entry) => entry.id),
    ["RUSTSEC-2020-0001", "RUSTSEC-2020-0002"],
  );
  assert.deepEqual(entries[0].justification, ["why: reason", "tracking: #1"]);
  assert.deepEqual(entries[1].justification, [], "a blank line resets the comment block");
});

test("every ignored advisory names why Vize is unaffected and what removes the waiver", () => {
  const config = readRepoFile(".cargo", "audit.toml");
  const entries = ignoredAdvisories(config);

  const seen = new Set<string>();
  for (const { id, justification } of entries) {
    assert.equal(seen.has(id), false, `${id} is ignored twice`);
    seen.add(id);

    assert.ok(
      justification.some((line) => line.startsWith("why:")),
      `${id} needs a "why:" line stating why the vulnerable path is unreachable in Vize`,
    );
    assert.ok(
      justification.some((line) => /^tracking: #\d+$/.test(line)),
      `${id} needs a "tracking: #<issue>" line pointing at the issue that removes it`,
    );
  }
});

test("the audit gate itself stays strict while advisories are waived", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");

  assert.match(
    workflow,
    /run: cargo audit --deny warnings\n/,
    "security-audit must keep failing on warnings; waivers belong in .cargo/audit.toml",
  );
  assert.doesNotMatch(
    workflow,
    /cargo audit[^\n]*--ignore/,
    "advisories are waived in .cargo/audit.toml, where the justification test can see them",
  );
});
