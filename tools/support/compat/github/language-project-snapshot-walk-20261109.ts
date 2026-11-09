import { createHash } from "node:crypto";
import { gunzipSync } from "node:zlib";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
type Identity = { path: string; bytes: number; sha256: string };
type Transition = {
  path: string;
  before: Omit<Identity, "path">;
  after: Omit<Identity, "path">;
  gzip: Identity;
  edits: Array<{ before: string; after: string }>;
};
type Custody = {
  schema: string;
  qualificationCredit: boolean;
  predecessors: Identity[];
  transitions: Transition[];
};

export const SNAPSHOT_WALK_CUSTODY =
  "tests/_fixtures/tooling/language-project-snapshot-walk-20261109/custody.json";
const recordHash = "6829d7c2b582eb34b332b288199a1adb6d72a37397ff11940a2905dbe2d17a3e";
const currentSources = [
  [
    "npm/typescript-plugin/source-documents.cjs",
    "a8ae69116d28c76893fd25da478b7bd2bd979768f18ffe4569494be3a3eaaf68",
  ],
  [
    "npm/typescript-plugin/virtual-modules.cjs",
    "9f497680a9c93895b286658d1690e00daa5add2ff95d9d85a31533788dea7d9b",
  ],
] as const;
const predecessorPaths = [
  "tests/_fixtures/tooling/language-project-demanded-sfc-20261105/custody.json",
];
const digest = (body: Buffer) => createHash("sha256").update(body).digest("hex");
function requireCurrent(condition: unknown): asserts condition {
  if (!condition)
    throw new Error("invalid classification successor: current source custody differs");
}
function pinned(read: ReadFile, identity: Identity): Buffer {
  const body = read("head", identity.path);
  requireCurrent(body && body.length === identity.bytes && digest(body) === identity.sha256);
  return body;
}

/** Authenticate the per-snapshot import walk transition before older replay.
 * It restores the exact demanded-SFC sources and grants no behavioral parity.
 */
export function historicalSnapshotWalkSource(
  read: ReadFile,
  side: "base" | "head",
  path: string,
  body: Buffer | null,
): Buffer | null {
  const source = currentSources.find(([name]) => name === path);
  if (side !== "head" || !body || !source || digest(body) !== source[1]) return body;
  const raw = read("head", SNAPSHOT_WALK_CUSTODY);
  requireCurrent(raw && digest(raw) === recordHash);
  const custody: Custody = JSON.parse(raw.toString("utf8"));
  requireCurrent(custody.schema === "vize.languageProjectSnapshotWalkSources.v1");
  requireCurrent(custody.qualificationCredit === false);
  requireCurrent(
    JSON.stringify(custody.predecessors.map((entry) => entry.path)) ===
      JSON.stringify(predecessorPaths),
  );
  for (const predecessor of custody.predecessors) pinned(read, predecessor);
  requireCurrent(
    JSON.stringify(custody.transitions.map((entry) => entry.path)) ===
      JSON.stringify(currentSources.map(([name]) => name)),
  );
  const entry = custody.transitions.find((entry) => entry.path === path)!;
  requireCurrent(body.length === entry.after.bytes && digest(body) === entry.after.sha256);
  const original = gunzipSync(pinned(read, entry.gzip), { maxOutputLength: entry.before.bytes });
  requireCurrent(
    original.length === entry.before.bytes && digest(original) === entry.before.sha256,
  );
  requireCurrent(Array.isArray(entry.edits) && entry.edits.length > 0);
  let forward = original.toString("utf8");
  for (const edit of entry.edits) {
    requireCurrent(
      edit.before.length > 0 &&
        edit.after.length > 0 &&
        edit.before !== edit.after &&
        forward.split(edit.before).length === 2,
    );
    forward = forward.replace(edit.before, edit.after);
  }
  requireCurrent(Buffer.from(forward).equals(body));
  let inverse = body.toString("utf8");
  for (const edit of [...entry.edits].reverse()) {
    requireCurrent(inverse.split(edit.after).length === 2);
    inverse = inverse.replace(edit.after, edit.before);
  }
  requireCurrent(Buffer.from(inverse).equals(original));
  return original;
}
