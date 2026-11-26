import assert from "node:assert/strict";
import { posix } from "node:path";
import { test } from "node:test";
import { cycleOwner } from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import type {
  CorpusNode,
  MetadataEntry,
  RecaptureBasis,
} from "../../tools/support/compat/github/canonical-corpus-types.ts";
import {
  at,
  blob,
  error,
  fixture,
  qualify,
  type RecaptureContext,
  type RecaptureFixture,
} from "./support/canonical-corpus-recapture-fixture.ts";

type AuthoredNode =
  | { path: string; mode: "40000" | "160000" }
  | { path: string; mode: "100644" | "120000"; source: Buffer };
const success = (path: string, isDirectory: boolean, followed?: number): MetadataEntry => ({
  path,
  isDirectory,
  ...(followed === undefined ? {} : { followed }),
});

function addNode(context: RecaptureFixture, authored: AuthoredNode, observed = true): void {
  const owner = posix.dirname(authored.path) === "." ? "" : posix.dirname(authored.path);
  assert(context.children.has(owner), "Authored census node needs an existing parent");
  const directory = authored.mode === "40000" || authored.mode === "160000";
  const source = "source" in authored ? authored.source : undefined;
  const node: CorpusNode = {
    path: authored.path,
    mode: authored.mode,
    id: source ? blob(source) : "4b825dc642cb6eb9a060e54bf8d69288fbee4904",
    repository: cycleOwner.repository,
    revision: cycleOwner.revision,
    ...(authored.mode === "120000" ? { target: source!.toString("utf8") } : {}),
  };
  assert(!context.nodes.has(authored.path), "Duplicate authored census node");
  context.nodes.set(authored.path, node);
  context.children.get(owner)!.add(posix.basename(authored.path));
  if (directory) context.children.set(authored.path, new Set());
  if (!observed) return;
  assert(context.baseline.raw.physical);
  context.baseline.raw.physical.push({
    path: authored.path,
    mode: directory ? 0o40755 : authored.mode === "120000" ? 0o120777 : 0o100644,
    size: source?.length ?? 4096,
    mtimeMs: 1,
    ctimeMs: 2,
    ...(authored.mode === "120000" ? { targetBase64: source!.toString("base64") } : {}),
  });
}

function censusFixture(): RecaptureContext {
  const context = fixture();
  addNode(context, { path: at(0, ".empty"), mode: "40000" });
  addNode(context, {
    path: at(0, ".gitignore"),
    mode: "100644",
    source: Buffer.from("ignored-cache/\n"),
  });
  addNode(context, { path: at(0, ".gitlink"), mode: "160000" });
  addNode(context, { path: "z-notes", mode: "40000" });
  addNode(context, { path: "z-notes/link", mode: "120000", source: Buffer.from("readme.txt") });
  addNode(context, { path: "z-notes/readme.txt", mode: "100644", source: Buffer.from("notes\n") });
  assert(context.baseline.raw.metadata);
  const starts = new Map(Array.from({ length: 4 }, (_, depth) => [at(depth, ".git"), depth]));
  context.baseline.raw.metadata = context.baseline.raw.metadata.flatMap((row) => {
    const depth = starts.get(row.path);
    return depth === undefined
      ? [row]
      : [
          success(at(depth, ".empty"), true, depth === 0 ? 0 : undefined),
          row,
          success(at(depth, ".gitignore"), false),
          success(at(depth, ".gitlink"), true, depth === 0 ? 0 : undefined),
        ];
  });
  context.baseline.raw.metadata.push(
    success("z-notes", true, 0),
    success("z-notes/link", false),
    success("z-notes/readme.txt", false),
  );
  return {
    ...context,
    raw: structuredClone(context.baseline.raw),
    boundary: structuredClone(context.baseline.boundary),
  };
}

function omitBothObservations(
  context: RecaptureContext,
  physicalPath: string,
  logicalPaths: readonly string[],
): void {
  const omitted = new Set(logicalPaths);
  for (const observation of [context.baseline.raw, context.raw]) {
    assert(observation.physical && observation.metadata);
    observation.physical = observation.physical.filter((row) => row.path !== physicalPath);
    observation.metadata = observation.metadata.filter((row) => !omitted.has(row.path));
    observation.rejected = observation.rejected.filter((row) => !omitted.has(row.path));
  }
  context.baseline.boundary.rejected = context.baseline.raw.rejected;
  context.boundary.rejected = context.raw.rejected;
  assert(context.nodes.has(physicalPath), "The authenticated graph must retain the missing entry");
  assert.deepEqual(context.raw.physical, context.baseline.raw.physical);
  assert.deepEqual(context.raw.metadata, context.baseline.raw.metadata);
}

function vueVectors(context: RecaptureContext): Pick<RecaptureBasis, "files"> & {
  original: RecaptureBasis["raw"]["files"];
  recaptured: RecaptureBasis["raw"]["files"];
} {
  return structuredClone({
    files: context.baseline.files,
    original: context.baseline.raw.files,
    recaptured: context.raw.files,
  });
}

test("complete physical census includes committed non-Vue files, links and empty directories", () => {
  const context = censusFixture();
  const result = qualify(context);
  assert.deepEqual(
    result.files,
    context.baseline.files.map(([path, , , id]) => [path, id]),
  );
  assert.equal(context.baseline.raw.files.length, 11);
  assert.equal(context.raw.files.length, 11);
});

for (const [name, path, aliases] of [
  [
    "non-Vue regular leaf",
    at(0, ".gitignore"),
    Array.from({ length: 4 }, (_, depth) => at(depth, ".gitignore")),
  ],
  ["non-Vue symlink", "z-notes/link", ["z-notes/link"]],
  [
    "empty directory",
    at(0, ".empty"),
    Array.from({ length: 4 }, (_, depth) => at(depth, ".empty")),
  ],
  [
    "empty mode-160000 entry",
    at(0, ".gitlink"),
    Array.from({ length: 4 }, (_, depth) => at(depth, ".gitlink")),
  ],
] as const) {
  test(`shared deletion of committed ${name} and every logical metadata row still fails`, () => {
    const context = censusFixture();
    qualify(context);
    const vectors = vueVectors(context);
    omitBothObservations(context, path, aliases);
    assert.deepEqual(
      vueVectors(context),
      vectors,
      "Every original and finite Vue tuple remains intact",
    );
    assert.throws(() => qualify(context), /Missing committed physical entry/u);
  });
}

test("deleting a shared non-Vue symlink and all its metadata/refusals cannot mask absence", () => {
  const context = censusFixture();
  addNode(context, { path: at(0, ".trace"), mode: "120000", source: Buffer.from(".gitignore") });
  const starts = new Map(Array.from({ length: 4 }, (_, depth) => [at(depth, "A.vue"), depth]));
  assert(context.baseline.raw.metadata);
  context.baseline.raw.metadata = context.baseline.raw.metadata.flatMap((row) => {
    const depth = starts.get(row.path);
    if (depth === undefined) return [row];
    const link = at(depth, ".trace");
    return [depth === 3 ? error(link, 4) : success(link, false), row];
  });
  context.baseline.raw.rejected = context.baseline.raw.metadata
    .filter((row) => row.error)
    .map((row) => ({ path: row.path, code: "ELOOP", followed: row.followed! }));
  context.baseline.boundary.rejected = context.baseline.raw.rejected;
  context.raw = structuredClone(context.baseline.raw);
  context.boundary = structuredClone(context.baseline.boundary);
  qualify(context);
  const vectors = vueVectors(context);
  const aliases = Array.from({ length: 4 }, (_, depth) => at(depth, ".trace"));
  omitBothObservations(context, at(0, ".trace"), aliases);
  assert(context.raw.rejected.every((row) => !aliases.includes(row.path)));
  assert.deepEqual(vueVectors(context), vectors);
  // The strict initial boundary comparison may refuse before the census check.
  assert.throws(() => qualify(context), Error);
});

test("exact excluded components and all their descendants may lack physical observations", () => {
  const context = censusFixture();
  for (const excluded of ["node_modules", "_git-worktrees"]) {
    addNode(context, { path: at(0, excluded), mode: "40000" }, false);
    addNode(context, { path: at(0, `${excluded}/nested`), mode: "40000" }, false);
    addNode(
      context,
      {
        path: at(0, `${excluded}/nested/Excluded.vue`),
        mode: "100644",
        source: Buffer.from("<template>excluded</template>\n"),
      },
      false,
    );
    addNode(
      context,
      {
        path: at(0, `${excluded}/nested/link`),
        mode: "120000",
        source: Buffer.from("Excluded.vue"),
      },
      false,
    );
  }
  const vectors = vueVectors(context);
  qualify(context);
  assert.deepEqual(vueVectors(context), vectors);
});

test("similarly named components retain the complete physical census requirement", () => {
  for (const name of ["node_modules-copy", "_git-worktrees-copy"]) {
    const context = censusFixture();
    addNode(context, { path: at(0, name), mode: "40000" }, false);
    addNode(
      context,
      {
        path: at(0, `${name}/readme.txt`),
        mode: "100644",
        source: Buffer.from("committed\n"),
      },
      false,
    );
    assert.throws(() => qualify(context), /Missing committed physical entry/u, name);
  }
});
