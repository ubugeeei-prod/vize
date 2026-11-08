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
  currentFiles: Identity[];
  transitions: Transition[];
};

export const DEMANDED_SFC_CUSTODY =
  "tests/_fixtures/tooling/language-project-demanded-sfc-20261105/custody.json";
const recordHash = "484d3383862410b07b974d9d43fa3fe52dad86b33a2a0687e79ee5ded31977b5";
const currentSources = [
  [
    "npm/typescript-plugin/language-service.cjs",
    "98463aad4e487a0f48521a0441f935a13a19db7144e310465c122fb5321efbee",
  ],
  [
    "npm/typescript-plugin/source-documents.cjs",
    "db1a5492e86be4ed594025a30b1cbc8c1d28b7f1090ad3d69019af5d15d1deed",
  ],
  [
    "npm/typescript-plugin/virtual-modules.cjs",
    "310a0a2dd0722953aa93fdd9645125b4a86e9af393aecd98c98d13df550c0fe1",
  ],
  [
    "tests/typescript-plugin/packed-consumer.ts",
    "11e53553055f6bfcd8b9f79e0fdecfc48ae728b18bc5f0c07698670d76517a1b",
  ],
] as const;
const predecessorPaths = [
  "tests/_fixtures/tooling/language-project-current-20261105/custody.json",
  "tests/_fixtures/tooling/language-project-current-20261105/plugin-source-successor.json",
];
const helperPath = "npm/typescript-plugin/project-options.cjs";
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
function before(read: ReadFile, entry: Transition): Buffer {
  const body = gunzipSync(pinned(read, entry.gzip), { maxOutputLength: entry.before.bytes });
  requireCurrent(body.length === entry.before.bytes && digest(body) === entry.before.sha256);
  return body;
}

/** Authenticate this complete production transition before composing older replay.
 * The original records, archives and their identities remain unchanged. This
 * grants historical source replay only; packed consumers qualify real behavior.
 */
export function historicalDemandedSfcSource(
  read: ReadFile,
  side: "base" | "head",
  path: string,
  body: Buffer | null,
): Buffer | null {
  const source = currentSources.find(([name]) => name === path);
  if (side !== "head" || !body || !source || digest(body) !== source[1]) return body;
  const raw = read("head", DEMANDED_SFC_CUSTODY);
  requireCurrent(raw && digest(raw) === recordHash);
  const custody: Custody = JSON.parse(raw.toString("utf8"));
  requireCurrent(custody.schema === "vize.languageProjectDemandedSfcSources.v1");
  requireCurrent(custody.qualificationCredit === false);
  requireCurrent(
    JSON.stringify(custody.predecessors.map((entry) => entry.path)) ===
      JSON.stringify(predecessorPaths),
  );
  for (const predecessor of custody.predecessors) pinned(read, predecessor);
  requireCurrent(
    JSON.stringify(custody.currentFiles.map((entry) => entry.path)) ===
      JSON.stringify([helperPath]),
  );
  requireCurrent(
    JSON.stringify(custody.transitions.map((entry) => entry.path)) ===
      JSON.stringify(currentSources.map(([name]) => name)),
  );
  const helper = pinned(read, custody.currentFiles[0]!);
  const virtual = before(read, custody.transitions[2]!).toString("utf8");
  const start = "// tsserver caches its extends resolution, including absent package candidates.\n";
  const end = "// One resolver snapshot belongs to the project, not to any individual SFC.\n";
  requireCurrent(virtual.split(start).length === 2 && virtual.split(end).length === 2);
  const moved = virtual.slice(virtual.indexOf(start), virtual.indexOf(end));
  requireCurrent(
    helper.equals(
      Buffer.from('"use strict";\n\n' + moved + "module.exports = { createProjectOptions };\n"),
    ),
  );
  const entry = custody.transitions.find((entry) => entry.path === path)!;
  requireCurrent(body.length === entry.after.bytes && digest(body) === entry.after.sha256);
  const original = before(read, entry);
  let forward = original.toString("utf8");
  requireCurrent(Array.isArray(entry.edits) && entry.edits.length > 0);
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
