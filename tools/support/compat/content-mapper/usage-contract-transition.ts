/** Advance helper custody across the shared usage contract (#12431). */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

export const usageContractRecord = "tests/_fixtures/tooling/usage-contract-12431.json";
export const usageContractRecordSha256 =
  "f085f1bd6949db53fea8b0af056c7841cd6dd90dd27ad4bb5bd3c316014ac0c6";

type Identity = { bytes: number; sha256: string };
type Hunk = { before: string; after: string };
interface Transition {
  schema: string;
  issue: number;
  credit: { compiled: boolean; native: boolean; performance: boolean };
  files: Array<{ path: string; before: Identity; after: Identity; hunks: Hunk[] }>;
}

const asset = "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
const identity = (text: string): Identity => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});

let record: Transition | undefined;
function load(): Transition {
  if (record) return record;
  const raw = readFileSync(new URL(`../../../../${usageContractRecord}`, import.meta.url), "utf8");
  assert.equal(
    identity(raw).sha256,
    usageContractRecordSha256,
    "immutable usage-contract transition",
  );
  const parsed = JSON.parse(raw) as Transition;
  assert.equal(parsed.schema, "vize.usageContractSource.v1");
  assert.equal(parsed.issue, 12431);
  assert.deepEqual(parsed.credit, { compiled: false, native: false, performance: false });
  record = parsed;
  return parsed;
}

/** The asset edit: `$UsageProps` and `$UsageCheckExtras` declared after `$PropValueOr`. */
export function usageContractEdit(): Hunk {
  const file = load().files.find((entry) => entry.path === asset);
  assert.ok(file && file.hunks.length === 1);
  return file.hunks[0]!;
}

/** The lines the asset gained, without their `export`. */
export function usageContractDeclarations(): string[] {
  const { before, after } = usageContractEdit();
  return after
    .slice(before.length)
    .split("\n")
    .filter(Boolean)
    .map((line) => line.replace(/^export /u, ""));
}

/** Advance a preamble image across the declarations; it must hold the predecessor line once. */
export function applyUsageContractIn(previous: string): string {
  const edit = usageContractEdit();
  assert.equal(previous.split(edit.before).length, 2, "usage-contract predecessor line");
  assert.ok(!previous.includes("$UsageProps<"), "the usage contract is not yet declared");
  return previous.replace(edit.before, () => edit.after);
}

/** Inverse of {@link applyUsageContractIn}. */
export function beforeUsageContractIn(current: string): string {
  const edit = usageContractEdit();
  assert.equal(current.split(edit.after).length, 2, "usage-contract current lines");
  return current.replace(edit.after, () => edit.before);
}

/** The exact pre-addition bytes of a recorded helper's current image; others pass through. */
export function beforeUsageContract(path: string, current: string): string {
  const file = load().files.find((entry) => entry.path === path);
  if (!file || identity(current).sha256 !== file.after.sha256) return current;
  for (const hunk of file.hunks) {
    assert.equal(current.split(hunk.after).length, 2, `usage-contract hunk: ${path}`);
    current = current.replace(hunk.after, () => hunk.before);
  }
  assert.deepEqual(identity(current), file.before, `usage-contract predecessor: ${path}`);
  return current;
}
