import assert from "node:assert/strict";

import type { LintSnapshotAlignment } from "./lint-snapshot-alignment.ts";
import {
  admitRefreshArchive,
  assertCheckedInTails,
  type Packet,
  type Support,
  withoutKeys,
} from "./lint-snapshot-app-e2e-refresh.ts";

const RULE = "vue/component-definition-name-casing";
// Nuxt's component scanner: one optional `.client`/`.server`, then any number
// of `.global`/`.island`, at the end of the file stem.
const MODE_SUFFIX = /(?:\.(?:client|server))?(?:\.global|\.island)*$/;
const PASCAL_CASE = /^[A-Z][A-Za-z0-9]*$/;
const KEBAB_CASE = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)+$/;

function stems(file: string) {
  const stem = file.replace(/^.*[\\/]/, "").replace(/(?:\.vue)+$/, "");
  return { stem, name: stem.replace(MODE_SUFFIX, "") };
}

// The only change this link admits: casing findings on files whose name is
// valid once Nuxt's mode suffix is removed disappear, and each packet's
// warning count drops by the same amount. Every other byte stays.
export function verifyModeSuffixPolicy(
  project: string,
  previousRaw: string,
  currentRaw: string,
  canonical: Support["canonical"],
) {
  const before = JSON.parse(previousRaw) as Packet[];
  const after = JSON.parse(currentRaw) as Packet[];
  assert.equal(after.length, before.length);
  let removedTotal = 0;
  for (const [index, beforePacket] of before.entries()) {
    const afterPacket = after[index];
    assert.deepEqual(
      withoutKeys(afterPacket, ["messages", "warningCount"]),
      withoutKeys(beforePacket, ["messages", "warningCount"]),
      "the refresh changes only reviewed diagnostics and warning counts",
    );
    const { stem, name } = stems(beforePacket.file);
    const suffixed = name !== stem && (PASCAL_CASE.test(name) || KEBAB_CASE.test(name));
    const kept = afterPacket.messages.map(canonical);
    let cursor = 0;
    let removed = 0;
    for (const message of beforePacket.messages) {
      if (cursor < kept.length && kept[cursor] === canonical(message)) {
        cursor++;
        continue;
      }
      assert.ok(suffixed, `${beforePacket.file}: only a Nuxt mode-suffixed file loses findings`);
      assert.deepEqual(
        withoutKeys(message, ["line", "column", "endLine", "endColumn", "help"]),
        {
          ruleId: RULE,
          ruleDocsPath: "docs/content/rules/reference/vue-component-definition-name-casing.md",
          severity: 1,
          message: `[vize:${RULE}] Component file name '${stem}' should be PascalCase or kebab-case`,
        },
        `${beforePacket.file}: only its file-name casing finding is removed`,
      );
      removed++;
    }
    assert.equal(cursor, kept.length, `${beforePacket.file}: no finding is added or reordered`);
    assert.ok(
      !suffixed || afterPacket.messages.every((message) => message.ruleId !== RULE),
      `${beforePacket.file}: a valid mode-suffixed name keeps no casing finding`,
    );
    assert.equal(afterPacket.warningCount, beforePacket.warningCount - removed);
    removedTotal += removed;
  }
  const expected: Record<string, number> = { elk: 7, "npmx.dev": 12 };
  assert.equal(removedTotal, expected[project], `${project} removed casing findings`);
}

export function admitNuxtModeSuffixRefresh(
  root: string,
  alignments: Map<string, LintSnapshotAlignment>,
  support: Support,
) {
  admitRefreshArchive(root, alignments, support, {
    directory: "2026-11-26-nuxt-mode-suffixes",
    predecessorCommit: "b7dfe22573ae4b35a08e292efb4d68ec335222eb",
    snapshotCommit: "e32ecc61eec1bf4fc9da3e9510fd4364a2c33a3f",
    entries: 2,
    policy: (project, previousRaw, currentRaw) =>
      verifyModeSuffixPolicy(project, previousRaw, currentRaw, support.canonical),
  });
  // This is the newest link, so the checked-in snapshots must equal its tails.
  assertCheckedInTails(root, alignments, support);
}
