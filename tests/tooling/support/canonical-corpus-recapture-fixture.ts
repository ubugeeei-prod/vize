import { createHash } from "node:crypto";
import { posix } from "node:path";
import { cycleOwner } from "../../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import { qualifyRecapturedObservation } from "../../../tools/support/compat/github/canonical-corpus-recapture.ts";
import type {
  Boundary,
  CorpusChildren,
  CorpusNodes,
  MetadataEntry,
  NativeObservation,
  PhysicalEntry,
  RecaptureBasis,
  VueFile,
} from "../../../tools/support/compat/github/canonical-corpus-types.ts";

export interface RecaptureFixture {
  nodes: CorpusNodes;
  children: CorpusChildren;
  baseline: RecaptureBasis;
}

export interface RecaptureContext extends RecaptureFixture {
  raw: NativeObservation;
  boundary: Boundary;
}

const bytes = {
  A: Buffer.from("<template>first\r\n</template>\r\n"),
  B: Buffer.from("<template>second𐀀</template>\n"),
};
export const blob = (source: Buffer) =>
  createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex");
export const at = (depth: number, suffix: string) =>
  `jellyfin-vue/${"packaging/deb/root/".repeat(depth)}${suffix}`;
export const error = (path: string, followed: number): MetadataEntry => ({
  path,
  followed,
  error: {
    code: "ELOOP",
    errno: -40,
    syscall: "stat",
    path: `/auth/corpus/${path}`,
    message: `ELOOP: too many symbolic links encountered, stat '/auth/corpus/${path}'`,
  },
});
export const vue = (path: string, source: "A" | "B"): VueFile => [
  path,
  createHash("sha256").update(bytes[source]).digest("hex"),
  bytes[source].length,
  blob(bytes[source]),
];

export function fixture(): RecaptureFixture {
  const nodes: CorpusNodes = new Map([["", { path: "", mode: "40000" }]]);
  const children: CorpusChildren = new Map([["", new Set()]]);
  const physical: PhysicalEntry[] = [];
  function add(path: string, mode: string, source?: Buffer) {
    const owner = posix.dirname(path) === "." ? "" : posix.dirname(path);
    const target = mode === "120000" ? source!.toString("utf8") : undefined;
    nodes.set(path, {
      path,
      mode,
      repository: cycleOwner.repository,
      revision: cycleOwner.revision,
      ...(source ? { id: blob(source) } : {}),
      ...(target ? { target } : {}),
    });
    children.get(owner)!.add(posix.basename(path));
    if (mode === "40000") children.set(path, new Set());
    physical.push({
      path,
      mode: mode === "40000" ? 0o40755 : mode === "120000" ? 0o120777 : 0o100644,
      size: source?.length ?? 4096,
      mtimeMs: 1,
      ctimeMs: 2,
      ...(target ? { targetBase64: source!.toString("base64") } : {}),
    });
  }
  add("jellyfin-vue", "40000");
  physical.push({ path: "jellyfin-vue/.git", mode: 0o100644, size: 20, mtimeMs: 1, ctimeMs: 2 });
  add(at(0, "A.vue"), "100644", bytes.A);
  add(at(0, "copy"), "120000", Buffer.from("sub"));
  add(at(0, "packaging"), "40000");
  add(at(0, "packaging/deb"), "40000");
  add(cycleOwner.path, "120000", Buffer.from(cycleOwner.target));
  add(at(0, "sub"), "40000");
  add(at(0, "sub/B.vue"), "100644", bytes.B);
  const metadata: MetadataEntry[] = [{ path: "jellyfin-vue", isDirectory: true, followed: 0 }];
  const success = (path: string, isDirectory: boolean, followed?: number) =>
    metadata.push({ path, isDirectory, ...(followed === undefined ? {} : { followed }) });
  // Authored layout oracle: four ancestor aliases, one one-hop copy per
  // reachable level, and the fourth owned link refused. No host IO is used.
  function frame(depth: number) {
    success(at(depth, ".git"), false);
    success(at(depth, "A.vue"), false, depth);
    if (depth === 3) metadata.push(error(at(depth, "copy"), 4));
    else {
      success(at(depth, "copy"), true, depth === 0 ? 1 : undefined);
      success(at(depth, "copy/B.vue"), false, depth + 1);
    }
    success(at(depth, "packaging"), true, depth === 0 ? 0 : undefined);
    success(at(depth, "packaging/deb"), true, depth === 0 ? 0 : undefined);
    if (depth === 3) metadata.push(error(at(depth, "packaging/deb/root"), 4));
    else {
      success(at(depth, "packaging/deb/root"), true, depth === 0 ? 1 : undefined);
      frame(depth + 1);
    }
    success(at(depth, "sub"), true, depth === 0 ? 0 : undefined);
    success(at(depth, "sub/B.vue"), false, depth);
  }
  frame(0);
  const rejected = [
    { path: at(3, "copy"), code: "ELOOP", followed: 4 },
    { path: at(3, "packaging/deb/root"), code: "ELOOP", followed: 4 },
  ];
  const boundary: Boundary = { code: "ELOOP", firstRejected: 4, rejected };
  const original = [
    vue(at(0, "A.vue"), "A"),
    vue(at(0, "copy/B.vue"), "B"),
    vue(at(1, "A.vue"), "A"),
    vue(at(1, "copy/B.vue"), "B"),
    vue(at(2, "A.vue"), "A"),
    vue(at(2, "copy/B.vue"), "B"),
    vue(at(3, "A.vue"), "A"),
    vue(at(3, "sub/B.vue"), "B"),
    vue(at(2, "sub/B.vue"), "B"),
    vue(at(1, "sub/B.vue"), "B"),
    vue(at(0, "sub/B.vue"), "B"),
  ];
  const files = [original[0], original[1], original[2], original[3], original[9], original[10]];
  const raw: NativeObservation = {
    physical,
    metadata,
    files: original,
    rejected,
    successfulFollows: 3,
  };
  return { nodes, children, baseline: { raw, boundary, files } };
}

export function recapture(): RecaptureContext {
  const context = fixture();
  const raw = structuredClone(context.baseline.raw);
  // A readable Vue with refused logical metadata remains in the full vector.
  raw.metadata![raw.metadata!.findIndex((row) => row.path === at(2, "A.vue"))] = error(
    at(2, "A.vue"),
    2,
  );
  // A refused directory removes exactly its descendants from this original
  // observation, while the independently authored finite vector stays complete.
  raw.metadata = raw
    .metadata!.map((row) => (row.path === at(2, "sub") ? error(row.path, 2) : row))
    .filter((row) => row.path !== at(2, "sub/B.vue"));
  raw.files = raw.files.filter(([path]) => path !== at(2, "sub/B.vue"));
  raw.rejected = [
    { path: at(2, "A.vue"), code: "ELOOP", followed: 2 },
    ...raw.rejected,
    { path: at(2, "sub"), code: "ELOOP", followed: 2 },
  ];
  const boundary = { ...context.baseline.boundary, rejected: raw.rejected };
  return { ...context, raw, boundary };
}
export const qualify = (
  context: RecaptureContext,
): ReturnType<typeof qualifyRecapturedObservation> =>
  qualifyRecapturedObservation(
    context.nodes,
    context.children,
    context.baseline,
    context.raw,
    context.boundary,
  );
