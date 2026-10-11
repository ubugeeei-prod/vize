import assert from "node:assert/strict";
import { test } from "node:test";
import { apiRoot, repository } from "../../tools/support/release/typed_defaults/custody.mjs";
import {
  authenticateReservedFixes,
  reservedFixes,
} from "../../tools/support/release/typed_defaults/reserved_fixes.mjs";

const head = "c".repeat(40);
function provider() {
  const rows = new Map();
  for (const { pr, merge } of reservedFixes) {
    rows.set(`${apiRoot}/pulls/${pr}`, {
      id: pr,
      number: pr,
      merged: true,
      state: "closed",
      merged_at: "2026-10-11T18:37:08Z",
      merge_commit_sha: merge,
      base: { ref: "main", repo: { full_name: repository } },
      head: { repo: { full_name: repository } },
    });
    rows.set(`${apiRoot}/compare/${merge}...${head}`, {
      base_commit: { sha: merge },
      merge_base_commit: { sha: merge },
      status: "ahead",
    });
  }
  return {
    rows,
    read: async (url) => {
      assert(
        rows.has(url),
        "only the authored three own-repository fix and H ancestry reads are allowed",
      );
      return structuredClone(rows.get(url));
    },
  };
}

await test("reserved443 requires all three actual own-repository merges in H, including alias", async () => {
  assert.deepEqual(reservedFixes, [
    { pr: 8668, merge: "19e8fa700d3c79631166bcaed4e3cfe65dda1903" },
    { pr: 8677, merge: "0de4eb3969cf230c2464e0d1edf1734b0dc388b8" },
    { pr: 8684, merge: "38af95bc04d6a8872cc5f045d60412bcaf19f70f" },
  ]);
  const valid = provider();
  const observed = await authenticateReservedFixes(head, valid.read);
  assert.equal(observed.length, 3);
  for (const [index, { fix, ancestry }] of observed.entries()) {
    assert.equal(fix.number, reservedFixes[index].pr);
    assert.equal(ancestry.merge_base_commit.sha, reservedFixes[index].merge);
  }
  for (const { pr, merge } of reservedFixes) {
    for (const [name, mutate] of [
      [
        "unmerged",
        (fix) => {
          fix.merged = false;
        },
      ],
      [
        "still open",
        (fix) => {
          fix.state = "open";
        },
      ],
      [
        "missing merged timestamp",
        (fix) => {
          fix.merged_at = null;
        },
      ],
      [
        "foreign actual merge",
        (fix) => {
          fix.merge_commit_sha = "f".repeat(40);
        },
      ],
      [
        "foreign PR",
        (fix) => {
          fix.number++;
        },
      ],
      [
        "invalid PR identity",
        (fix) => {
          fix.id = 0;
        },
      ],
      [
        "foreign base branch",
        (fix) => {
          fix.base.ref = "unreviewed";
        },
      ],
      [
        "foreign base repository",
        (fix) => {
          fix.base.repo.full_name = "foreign/vize";
        },
      ],
      [
        "foreign head repository",
        (fix) => {
          fix.head.repo.full_name = "foreign/vize";
        },
      ],
    ]) {
      const f = provider();
      mutate(f.rows.get(`${apiRoot}/pulls/${pr}`));
      await assert.rejects(
        () => authenticateReservedFixes(head, f.read),
        undefined,
        `${pr}: ${String(name)}`,
      );
    }
    for (const [name, mutate] of [
      [
        "foreign base commit",
        (a) => {
          a.base_commit.sha = "f".repeat(40);
        },
      ],
      [
        "H omits this actual fix",
        (a) => {
          a.merge_base_commit.sha = "f".repeat(40);
        },
      ],
      [
        "diverged",
        (a) => {
          a.status = "diverged";
        },
      ],
      [
        "older H",
        (a) => {
          a.status = "behind";
        },
      ],
    ]) {
      const f = provider();
      mutate(f.rows.get(`${apiRoot}/compare/${merge}...${head}`));
      await assert.rejects(
        () => authenticateReservedFixes(head, f.read),
        undefined,
        `${pr}: ${String(name)}`,
      );
    }
  }
  for (const malformed of [undefined, "main", head.toUpperCase(), "c".repeat(39)])
    await assert.rejects(() => authenticateReservedFixes(malformed, valid.read));
});
