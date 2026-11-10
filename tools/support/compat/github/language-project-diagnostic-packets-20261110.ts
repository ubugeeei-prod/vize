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

export const DIAGNOSTIC_PACKETS_CUSTODY =
  "tests/_fixtures/tooling/language-project-diagnostic-packets-20261110/custody.json";
const recordHash = "64b8ac5f5b68e34ec1dabe4873d722ed2fb15780046eaedc28fb4b25d32848c0";
const currentSources = [
  [
    "npm/typescript-plugin/language-service.cjs",
    "6245cd30429d0fe2e5b47507703c377cce4a337a6c7626dc8593c34b3dbec72c",
  ],
  [
    "npm/typescript-plugin/virtual-modules.cjs",
    "c91aa43f12f2c0ac30a8a4cb553af6b1cfa55692b44983381f20d7f81ded054a",
  ],
] as const;
const predecessorPaths = [
  "tests/_fixtures/tooling/language-project-snapshot-walk-20261109/custody.json",
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

/** Authenticate the shared diagnostic packet transition before older replay.
 * It restores the exact snapshot-walk and demanded-SFC sources and grants no
 * behavioral parity.
 */
export function historicalDiagnosticPacketsSource(
  read: ReadFile,
  side: "base" | "head",
  path: string,
  body: Buffer | null,
): Buffer | null {
  const source = currentSources.find(([name]) => name === path);
  if (side !== "head" || !body || !source || digest(body) !== source[1]) return body;
  const raw = read("head", DIAGNOSTIC_PACKETS_CUSTODY);
  requireCurrent(raw && digest(raw) === recordHash);
  const custody: Custody = JSON.parse(raw.toString("utf8"));
  requireCurrent(custody.schema === "vize.languageProjectDiagnosticPacketSources.v1");
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
