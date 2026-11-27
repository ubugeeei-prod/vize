import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

import type { LintSnapshotAlignment } from "./lint-snapshot-alignment.ts";

export type Packet = Record<string, any> & { file: string; messages: Record<string, any>[] };
export type Support = {
  canonical: (value: unknown) => string;
  hash: (value: string | Buffer) => string;
  verify: (previousRaw: string, currentRaw: string, review: LintSnapshotAlignment) => void;
};

export const withoutKeys = (value: Record<string, any>, keys: string[]) =>
  Object.fromEntries(Object.entries(value).filter(([key]) => !keys.includes(key)));

function verifyRefreshPolicy(
  project: string,
  previousRaw: string,
  currentRaw: string,
  canonical: Support["canonical"],
) {
  const before = JSON.parse(previousRaw) as Packet[];
  const after = JSON.parse(currentRaw) as Packet[];
  const semanticChanges = new Map<string, number>();
  const count = (name: string) => semanticChanges.set(name, (semanticChanges.get(name) ?? 0) + 1);
  for (const [packetIndex, beforePacket] of before.entries()) {
    const afterPacket = after[packetIndex];
    assert.equal(afterPacket.file, beforePacket.file);
    const remaining = [...afterPacket.messages];
    let added = 0;
    let removed = 0;
    for (const message of beforePacket.messages) {
      const exact = remaining.findIndex((candidate) => canonical(candidate) === canonical(message));
      if (exact !== -1) {
        remaining.splice(exact, 1);
        continue;
      }
      const presentation = remaining.findIndex(
        (candidate) =>
          canonical(withoutKeys(candidate, ["ruleDocsPath"])) ===
          canonical(withoutKeys(message, ["ruleDocsPath"])),
      );
      if (presentation !== -1) {
        const candidate = remaining.splice(presentation, 1)[0];
        assert.match(
          message.ruleDocsPath,
          /^docs\/content\/rules\/(?:accessibility|html|ssr|type-and-script|vue)\.md$/,
        );
        assert.equal(
          candidate.ruleDocsPath,
          `docs/content/rules/reference/${String(message.ruleId).replace("/", "-")}.md`,
        );
        continue;
      }
      count(`removed:${message.ruleId}`);
      removed++;
    }
    for (const message of remaining) {
      count(`added:${message.ruleId}`);
      added++;
    }
    assert.deepEqual(
      withoutKeys(afterPacket, ["messages", "warningCount"]),
      withoutKeys(beforePacket, ["messages", "warningCount"]),
      "the refresh changes only reviewed diagnostics and rule documentation paths",
    );
    assert.equal(afterPacket.warningCount, beforePacket.warningCount + added - removed);
  }
  const actual = Object.fromEntries(
    [...semanticChanges].sort(([left], [right]) => left.localeCompare(right)),
  );
  const expected: Record<string, Record<string, number>> = {
    "ant-design-vue": {
      "added:script/require-explicit-emits": 1,
      "added:vue/v-on-event-hyphenation": 40,
    },
    elk: {},
    misskey: {
      "added:vue/no-template-shadow": 11,
      "removed:vue/no-invalid-html-attribute": 1,
    },
    "npmx.dev": { "added:vue/no-template-shadow": 3 },
    "nuxt-ui": { "removed:a11y/no-static-element-interactions": 1 },
    "reka-ui": {
      "added:script/require-explicit-emits": 2,
      "removed:a11y/interactive-supports-focus": 2,
    },
    "vuefes-2025": {},
  };
  assert.deepEqual(actual, expected[project]);
}

export interface RefreshLink {
  directory: string;
  predecessorCommit: string;
  snapshotCommit: string;
  entries: number;
  policy: (project: string, previousRaw: string, currentRaw: string) => void;
}

// Appends one reviewed archive to the end of every chain it names. The
// checked-in bytes are compared by the last link only (`assertCheckedInTails`),
// because an older link's current snapshot is a later link's predecessor.
export function admitRefreshArchive(
  root: string,
  alignments: Map<string, LintSnapshotAlignment>,
  support: Support,
  link: RefreshLink,
) {
  const directory = path.join(root, "tests/_fixtures/_projects/lint-review", link.directory);
  const inventory = JSON.parse(fs.readFileSync(path.join(directory, "inventory.json"), "utf8"));
  assert.deepEqual(
    inventory.map((entry: { path: string }) => entry.path),
    ["snapshots.json.gz"],
  );
  const bytes = fs.readFileSync(path.join(directory, "snapshots.json.gz"));
  assert.equal(bytes.length, inventory[0].bytes);
  assert.equal(support.hash(bytes), inventory[0].sha256);
  const archive = JSON.parse(gunzipSync(bytes).toString());
  assert.equal(archive.schema, "vize-lint-snapshot-successor-v2");
  assert.equal(archive.predecessorCommit, link.predecessorCommit);
  assert.equal(archive.snapshotCommit, link.snapshotCommit);
  assert.equal(archive.entries.length, link.entries);
  for (const entry of archive.entries) {
    const rootReview = alignments.get(entry.path);
    assert.ok(rootReview);
    let tail = rootReview;
    while (tail.successor) tail = tail.successor.review;
    assert.equal(support.hash(entry.previousRaw), tail.currentSha256);
    assert.equal(support.hash(entry.currentRaw), entry.alignment.currentSha256);
    assert.equal(entry.alignment.previousSha256, tail.currentSha256);
    support.verify(entry.previousRaw, entry.currentRaw, entry.alignment);
    link.policy(entry.project, entry.previousRaw, entry.currentRaw);
    tail.successor = {
      previousRaw: entry.previousRaw,
      fixturePin: entry.fixturePin,
      review: entry.alignment,
    };
  }
}

export function assertCheckedInTails(
  root: string,
  alignments: Map<string, LintSnapshotAlignment>,
  support: Support,
) {
  for (const [snapshot, rootReview] of alignments) {
    let tail = rootReview;
    while (tail.successor) tail = tail.successor.review;
    assert.equal(
      support.hash(fs.readFileSync(path.join(root, snapshot), "utf8")),
      tail.currentSha256,
      `${snapshot} matches the end of its reviewed chain`,
    );
  }
}

export function admitAppE2eRefresh(
  root: string,
  alignments: Map<string, LintSnapshotAlignment>,
  support: Support,
) {
  admitRefreshArchive(root, alignments, support, {
    directory: "2026-11-06-app-e2e-refresh",
    predecessorCommit: "698cc668042616b43dfb7027f741a5a99fb38adc",
    snapshotCommit: "9dc1c1f81b43b1b585d65068aa27800c1247f29f",
    entries: 7,
    policy: (project, previousRaw, currentRaw) =>
      verifyRefreshPolicy(project, previousRaw, currentRaw, support.canonical),
  });
}
