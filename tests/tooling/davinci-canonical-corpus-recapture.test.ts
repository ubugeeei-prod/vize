import assert from "node:assert/strict";
import { test } from "node:test";
import { cycleOwner } from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import { qualifyRecapturedObservation } from "../../tools/support/compat/github/canonical-corpus-recapture.ts";
import type {
  MetadataEntry,
  VueFile,
} from "../../tools/support/compat/github/canonical-corpus-types.ts";
import {
  at,
  blob,
  error,
  fixture,
  qualify,
  recapture,
  vue,
} from "./support/canonical-corpus-recapture-fixture.ts";

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
    assert.throws(() => qualify(context), Error, name);
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
      "shared physical source size",
      ({ raw, baseline }) => {
        for (const observation of [raw, baseline.raw])
          observation.physical!.find((row) => row.path === at(0, "A.vue"))!.size += 1;
      },
    ],
    [
      "shared unknown physical symlink",
      ({ raw, baseline }) => {
        for (const observation of [raw, baseline.raw]) {
          const entry = observation.physical!.find((row) => row.path === at(0, ".git"))!;
          entry.mode = 0o120777;
          entry.targetBase64 = Buffer.from("outside").toString("base64");
        }
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
    assert.throws(() => qualify(context), Error, name);
  }
});

test("an existing metadata refusal outside the finite cut cannot disappear", () => {
  const context = fixture();
  const plain = Buffer.from("plain source");
  const target = Buffer.from("Plain.txt");
  for (const [name, mode, source] of [
    ["Plain.txt", "100644", plain],
    ["Trace.txt", "120000", target],
  ] as const) {
    const path = at(0, name);
    context.nodes.set(path, {
      path,
      mode,
      id: blob(source),
      repository: cycleOwner.repository,
      revision: cycleOwner.revision,
      ...(mode === "120000" ? { target: source.toString("utf8") } : {}),
    });
    context.children.get("jellyfin-vue")!.add(name);
    context.baseline.raw.physical!.push({
      path,
      mode: mode === "120000" ? 0o120777 : 0o100644,
      size: source.length,
      mtimeMs: 1,
      ctimeMs: 2,
      ...(mode === "120000" ? { targetBase64: source.toString("base64") } : {}),
    });
  }
  const metadata: MetadataEntry[] = [{ path: "jellyfin-vue", isDirectory: true, followed: 0 }];
  const original: VueFile[] = [];
  const success = (path: string, isDirectory: boolean, followed?: number) =>
    metadata.push({ path, isDirectory, ...(followed === undefined ? {} : { followed }) });
  // Three owned levels are independently authored. Trace is non-Vue, with one
  // additional finite link: its existing refusal at level one is outside cut.
  const frame = (depth: number) => {
    success(at(depth, ".git"), false);
    success(at(depth, "A.vue"), false, depth);
    original.push(vue(at(depth, "A.vue"), "A"));
    success(at(depth, "Plain.txt"), false);
    if (depth > 0) metadata.push(error(at(depth, "Trace.txt"), depth + 1));
    else success(at(depth, "Trace.txt"), false);
    if (depth === 2) metadata.push(error(at(depth, "copy"), 3));
    else {
      success(at(depth, "copy"), true, depth === 0 ? 1 : undefined);
      success(at(depth, "copy/B.vue"), false, depth + 1);
      original.push(vue(at(depth, "copy/B.vue"), "B"));
    }
    success(at(depth, "packaging"), true, depth === 0 ? 0 : undefined);
    success(at(depth, "packaging/deb"), true, depth === 0 ? 0 : undefined);
    if (depth === 2) metadata.push(error(at(depth, "packaging/deb/root"), 3));
    else {
      success(at(depth, "packaging/deb/root"), true, depth === 0 ? 1 : undefined);
      frame(depth + 1);
    }
    success(at(depth, "sub"), true, depth === 0 ? 0 : undefined);
    success(at(depth, "sub/B.vue"), false, depth);
    original.push(vue(at(depth, "sub/B.vue"), "B"));
  };
  frame(0);
  const rejected = metadata
    .filter((row) => row.error)
    .map((row) => ({
      path: row.path,
      code: "ELOOP",
      followed: row.followed!,
    }));
  context.baseline.raw.metadata = metadata;
  context.baseline.raw.files = original;
  context.baseline.raw.rejected = rejected;
  context.baseline.raw.successfulFollows = 2;
  context.baseline.boundary = { code: "ELOOP", firstRejected: 3, rejected };
  const before = structuredClone(context.baseline.raw);
  qualifyRecapturedObservation(
    context.nodes,
    context.children,
    context.baseline,
    before,
    context.baseline.boundary,
  );
  const removed = at(1, "Trace.txt");
  before.metadata![before.metadata!.findIndex((row) => row.path === removed)] = {
    path: removed,
    isDirectory: false,
  };
  before.rejected = before.rejected.filter((row) => row.path !== removed);
  assert.throws(
    () =>
      qualifyRecapturedObservation(context.nodes, context.children, context.baseline, before, {
        ...context.baseline.boundary,
        rejected: before.rejected,
      }),
    /outside repeated finite descendants/u,
  );
});
