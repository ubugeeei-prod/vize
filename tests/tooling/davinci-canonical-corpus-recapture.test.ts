import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { posix } from "node:path";
import { test } from "node:test";
import { cycleOwner } from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import { qualifyRecapturedObservation } from "../../tools/support/compat/github/canonical-corpus-recapture.ts";
import type {
  Boundary,
  CorpusChildren,
  CorpusNodes,
  MetadataEntry,
  NativeObservation,
  PhysicalEntry,
  VueFile,
} from "../../tools/support/compat/github/canonical-corpus-types.ts";

const bytes = {
  A: Buffer.from("<template>first\r\n</template>\r\n"),
  B: Buffer.from("<template>second𐀀</template>\n"),
};
const blob = (source: Buffer) =>
  createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex");
const at = (depth: number, suffix: string) =>
  `jellyfin-vue/${"packaging/deb/root/".repeat(depth)}${suffix}`;
const error = (path: string, followed: number): MetadataEntry => ({
  path,
  followed,
  error: { code: "ELOOP", errno: -40, syscall: "stat", path: `/auth/corpus/${path}` },
});
const vue = (path: string, source: "A" | "B"): VueFile => [
  path,
  createHash("sha256").update(bytes[source]).digest("hex"),
  bytes[source].length,
  blob(bytes[source]),
];

function fixture() {
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

function recapture() {
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
const qualify = (context: ReturnType<typeof recapture>) =>
  qualifyRecapturedObservation(
    context.nodes,
    context.children,
    context.baseline,
    context.raw,
    context.boundary,
  );

test("preserves distinct original observations while proving complete ordered finite bytes", () => {
  const context = recapture();
  const before = structuredClone(context);
  const result = qualify(context);
  assert.deepEqual(result, {
    cut: [at(1, "packaging")],
    files: context.baseline.files.map(([path, , , id]) => [path, id]),
  });
  assert.equal(context.baseline.raw.files.length, 11);
  assert.equal(context.raw.files.length, 10);
  assert(context.raw.files.some(([path]) => path === at(2, "A.vue")));
  assert.deepEqual(context, before, "qualification must not rewrite either observation");
});

test("unchanged strict recapture qualifies without relaxing its original boundary", () => {
  const { nodes, children, baseline } = fixture();
  const result = qualifyRecapturedObservation(
    nodes,
    children,
    baseline,
    structuredClone(baseline.raw),
    structuredClone(baseline.boundary),
  );
  assert.deepEqual(
    result.files,
    baseline.files.map(([path, , , id]) => [path, id]),
  );
});

test("refusals at cut node, first root, finite copy or outside repeated subtree fail", () => {
  for (const path of [
    at(1, "packaging"),
    cycleOwner.path,
    at(0, "copy"),
    at(1, "copy"),
    at(0, "sub"),
  ]) {
    const context = recapture();
    const row = context.raw.metadata!.find((row) => row.path === path)!;
    const followed =
      path === cycleOwner.path || path.endsWith("copy")
        ? path === at(1, "copy")
          ? 2
          : 1
        : path === at(1, "packaging")
          ? 1
          : 0;
    Object.assign(row, error(path, followed));
    delete row.isDirectory;
    context.raw.rejected.push({ path, code: "ELOOP", followed });
    assert.throws(
      () => qualify(context),
      /outside repeated finite descendants|Foreign recapture kernel refusal/,
      path,
    );
  }
});

test("all metadata and rejection rows must retain order, census, codes, syscall and counts", () => {
  const changes: Array<[string, (context: ReturnType<typeof recapture>) => void]> = [
    [
      "missing",
      ({ raw }) => {
        raw.metadata!.splice(1, 1);
      },
    ],
    [
      "duplicate",
      ({ raw }) => {
        raw.metadata!.splice(1, 0, structuredClone(raw.metadata![1]));
      },
    ],
    [
      "reordered",
      ({ raw }) => {
        [raw.metadata![1], raw.metadata![2]] = [raw.metadata![2], raw.metadata![1]];
      },
    ],
    [
      "extra",
      ({ raw }) => {
        raw.metadata!.push(structuredClone(raw.metadata![0]));
      },
    ],
    [
      "error code",
      ({ raw }) => {
        raw.metadata!.find((row) => row.error)!.error!.code = "ENOENT";
      },
    ],
    [
      "syscall",
      ({ raw }) => {
        raw.metadata!.find((row) => row.error)!.error!.syscall = "read";
      },
    ],
    [
      "errno",
      ({ raw }) => {
        raw.metadata!.find((row) => row.error)!.error!.errno = -1;
      },
    ],
    [
      "error path",
      ({ raw }) => {
        raw.metadata!.find((row) => row.error)!.error!.path = "/elsewhere";
      },
    ],
    [
      "follow count",
      ({ raw }) => {
        raw.metadata!.find((row) => row.error)!.followed! += 1;
      },
    ],
    [
      "directory classification",
      ({ raw }) => {
        raw.metadata![0].isDirectory = false;
      },
    ],
    [
      "raw rejection",
      ({ raw }) => {
        raw.rejected[0].followed += 1;
      },
    ],
    [
      "boundary census",
      ({ boundary }) => {
        boundary.rejected = boundary.rejected.slice(1);
      },
    ],
    [
      "successful maximum",
      ({ raw }) => {
        raw.successfulFollows -= 1;
      },
    ],
    [
      "owned witness",
      ({ boundary }) => {
        boundary.firstRejected -= 1;
      },
    ],
  ];
  for (const [name, mutate] of changes) {
    const context = recapture();
    mutate(context);
    assert.throws(() => qualify(context), undefined, name);
  }
});

test("finite and original bytes, aliases, modes and symlink owner cannot drift", () => {
  const changes: Array<[string, (context: ReturnType<typeof recapture>) => void]> = [
    [
      "missing finite source",
      ({ baseline }) => {
        baseline.files.pop();
      },
    ],
    [
      "reordered finite vector",
      ({ baseline }) => {
        baseline.files.reverse();
      },
    ],
    [
      "duplicate finite vector",
      ({ baseline }) => {
        baseline.files.push(structuredClone(baseline.files[0]));
      },
    ],
    [
      "original alias removed",
      ({ raw }) => {
        raw.files.splice(0, 1);
      },
    ],
    [
      "readable refused Vue removed",
      ({ raw }) => {
        raw.files = raw.files.filter(([path]) => path !== at(2, "A.vue"));
      },
    ],
    [
      "original aliases reordered",
      ({ raw }) => {
        raw.files.reverse();
      },
    ],
    [
      "changed sha",
      ({ raw }) => {
        raw.files[0][1] = "0".repeat(64);
      },
    ],
    [
      "changed size",
      ({ raw }) => {
        raw.files[0][2] += 1;
      },
    ],
    [
      "changed blob",
      ({ raw }) => {
        raw.files[0][3] = "0".repeat(40);
      },
    ],
    [
      "physical mode",
      ({ raw }) => {
        raw.physical![0].mode = 0o100644;
      },
    ],
    [
      "physical ctime",
      ({ raw }) => {
        raw.physical![0].ctimeMs += 1;
      },
    ],
    [
      "shared physical executable mode",
      ({ raw, baseline }) => {
        for (const observation of [raw, baseline.raw])
          observation.physical!.find((row) => row.path === at(0, "A.vue"))!.mode = 0o100755;
      },
    ],
    [
      "shared physical symlink target",
      ({ raw, baseline }) => {
        for (const observation of [raw, baseline.raw])
          observation.physical!.find((row) => row.path === cycleOwner.path)!.targetBase64 =
            Buffer.from("../other").toString("base64");
      },
    ],
    [
      "foreign cycle owner",
      ({ nodes }) => {
        nodes.get(cycleOwner.path)!.id = "f".repeat(40);
      },
    ],
    [
      "initial original vector",
      ({ baseline }) => {
        baseline.raw.files.pop();
      },
    ],
  ];
  for (const [name, mutate] of changes) {
    const context = recapture();
    mutate(context);
    assert.throws(() => qualify(context), undefined, name);
  }
});
