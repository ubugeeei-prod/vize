import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, posix } from "node:path";
import { test } from "node:test";
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { gitObjectId } from "../../tools/support/compat/github/canonical-corpus-inventory.mjs";
import {
  cycleOwner,
  kernelPrefixWitness,
  finiteCommittedInventory,
  pathOrder,
  resolveLogical,
  validateLogicalGraph,
  walkCommittedGraph,
} from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import {
  collectorHarness,
  collectFinitePhysical,
  collectNativePhysical,
  parseNativeVector,
  validateFiniteNativeVector,
} from "../../tools/support/compat/github/canonical-corpus-native-walk.ts";

// Authored small controls exercise the frozen original and current real Rust functions, actual
// local kernel boundary and raw bytes. They are not the licensed full corpus.
function fixture(root) {
  const nodes = new Map([["", { path: "", mode: "40000" }]]);
  const children = new Map([["", new Set()]]);
  const add = (path, mode, bytes) => {
    const parent = posix.dirname(path) === "." ? "" : posix.dirname(path);
    if (!nodes.has(parent)) add(parent, "40000");
    const node = { path, mode, repository: cycleOwner.repository, revision: cycleOwner.revision };
    if (mode === "40000") {
      children.set(path, new Set());
      mkdirSync(join(root, path), { recursive: true });
    } else {
      node.id = gitObjectId("blob", bytes);
      if (mode === "120000") {
        node.target = bytes.toString("utf8");
        symlinkSync(node.target, join(root, path));
      } else writeFileSync(join(root, path), bytes);
    }
    nodes.set(path, node);
    children.get(parent).add(posix.basename(path));
    return node;
  };
  add("jellyfin-vue/src/Z.vue", "100644", Buffer.from("<template>ASCII\r\n</template>\r\n"));
  add("jellyfin-vue/src/\ue000.vue", "100644", Buffer.from("<template>private-use</template>\n"));
  add("jellyfin-vue/src/𐀀.vue", "100644", Buffer.from("<template>astral</template>\n"));
  add("jellyfin-vue/src/.vue", "100644", Buffer.from("not a Rust extension match\n"));
  add(
    "jellyfin-vue/node_modules/Excluded.vue",
    "100644",
    Buffer.from("original dependency exclusion\n"),
  );
  add(cycleOwner.path, "120000", Buffer.from(cycleOwner.target));
  add("jellyfin-vue/copy", "120000", Buffer.from("src"));
  return { nodes, children, add };
}

await test("directory syscall exhaustion preserves the complete native logical vector and genuine metadata boundary", (context) => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-directory-syscalls-"));
  try {
    const { nodes, children } = fixture(temporary);
    const expected = collectNativePhysical(temporary, nodes, children);
    const readdir = fs.readdirSync;
    const rejected = join(
      temporary,
      "jellyfin-vue",
      ...Array(expected.boundary.firstRejected).fill("packaging/deb/root"),
    );
    let originalError;
    try {
      readdir(rejected);
    } catch (error) {
      originalError = error;
    }
    assert.equal(
      originalError?.code,
      "ELOOP",
      "The control witnesses an actual directory syscall limit",
    );
    const logicalDirectory = join(
      temporary,
      "jellyfin-vue/packaging/deb/root/packaging/deb/root/src",
    );
    context.mock.method(fs, "readdirSync", (directory, ...args) => {
      if (directory === logicalDirectory) {
        const error = new Error(originalError.message, { cause: originalError });
        Object.assign(error, {
          code: originalError.code,
          errno: originalError.errno,
          syscall: "scandir",
          path: directory,
        });
        throw error;
      }
      return readdir(directory, ...args);
    });
    syncBuiltinESMExports();
    assert.deepEqual(collectNativePhysical(temporary, nodes, children), expected);
  } finally {
    context.mock.restoreAll();
    syncBuiltinESMExports();
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("observation persistence failures retain the exact traversal error and fail closed", (context) => {
  const originalReaddir = fs.readdirSync;
  const originalRealpath = fs.realpathSync;
  const nodes = new Map([["", { path: "", mode: "40000" }]]);
  const children = new Map([["", new Set()]]);
  const traversalFailure = new Error("authored directory read failure");
  const persistenceFailure = new Error("authored observation write failure");
  const root = "/authored/no-filesystem-access";
  let observations = 0;
  try {
    const readdir = context.mock.method(fs, "readdirSync", () => {
      throw traversalFailure;
    });
    syncBuiltinESMExports();
    assert.throws(
      () =>
        collectNativePhysical(root, nodes, children, () => {
          observations += 1;
          throw persistenceFailure;
        }),
      (error) => {
        assert(error instanceof AggregateError);
        assert.deepEqual(error.errors, [traversalFailure, persistenceFailure]);
        return true;
      },
    );
    assert.equal(observations, 1);
    assert.throws(
      () => collectNativePhysical(root, nodes, children, () => (observations += 1)),
      (error) => error === traversalFailure,
    );
    assert.equal(observations, 2);
    readdir.mock.mockImplementation(() => []);
    context.mock.method(fs, "realpathSync", (path) => path);
    syncBuiltinESMExports();
    assert.throws(
      () =>
        collectNativePhysical(root, nodes, children, () => {
          observations += 1;
          throw persistenceFailure;
        }),
      (error) => error === persistenceFailure,
    );
    assert.equal(observations, 3);
  } finally {
    context.mock.restoreAll();
    syncBuiltinESMExports();
  }
  assert.equal(fs.readdirSync, originalReaddir);
  assert.equal(fs.realpathSync, originalRealpath);
});

await test("raw recapture rows survive an unchanged strict boundary refusal without replacing the first observation", (context) => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-boundary-evidence-"));
  try {
    const { nodes, children } = fixture(temporary);
    const retained = [];
    const observe = (raw) => retained.push(structuredClone(raw));
    const original = collectNativePhysical(temporary, nodes, children, observe);
    const originalBytes = JSON.stringify(retained[0]);
    assert.equal(retained.length, 1);
    assert.deepEqual(retained[0].files, original.files);
    assert.deepEqual(retained[0].rejected, original.boundary.rejected);
    assert.equal(retained[0].successfulFollows + 1, original.boundary.firstRejected);
    const stat = fs.statSync;
    const rejected = "jellyfin-vue/packaging/deb/root/copy";
    const failure = Object.assign(new Error("authored metadata refusal"), {
      code: "ELOOP",
      errno: -40,
      syscall: "stat",
      path: join(temporary, rejected),
    });
    // The finite alias cannot certify a kernel prefix. Its extra refusal must
    // fail the whole committed-path comparator while the real prefix is retained.
    assert.equal(kernelPrefixWitness(nodes, rejected, posix.dirname(rejected)), undefined);
    context.mock.method(fs, "statSync", (path, ...args) => {
      if (path === failure.path) throw failure;
      return stat(path, ...args);
    });
    syncBuiltinESMExports();
    assert.throws(
      () => collectNativePhysical(temporary, nodes, children, observe),
      (error) => {
        assert(error instanceof assert.AssertionError);
        assert.match(error.message, /^Committed kernel boundary paths changed(?:\n|$)/u);
        assert.deepEqual(error.actual, original.boundary.rejected);
        assert.deepEqual(error.expected, [
          { path: rejected, code: "ELOOP", followed: resolveLogical(nodes, rejected).followed },
          ...original.boundary.rejected,
        ]);
        return true;
      },
    );
    assert.equal(retained.length, 2);
    assert.equal(JSON.stringify(retained[0]), originalBytes);
    assert.deepEqual(retained[1].rejected, [
      { path: rejected, code: "ELOOP", followed: resolveLogical(nodes, rejected).followed },
      ...original.boundary.rejected,
    ]);
    assert.deepEqual(
      retained[1].metadata.find((row) => row.path === rejected),
      {
        path: rejected,
        error: {
          code: failure.code,
          errno: failure.errno,
          syscall: failure.syscall,
          path: failure.path,
          message: failure.message,
        },
        followed: resolveLogical(nodes, rejected).followed,
      },
    );
    assert.deepEqual(retained[1].physical, retained[0].physical);
    assert.deepEqual(
      retained[1].metadata.filter((row) => row.isDirectory && row.followed !== undefined),
      retained[0].metadata.filter(
        (row) =>
          row.isDirectory &&
          row.followed !== undefined &&
          row.path !== rejected &&
          !row.path.startsWith(`${rejected}/`),
      ),
    );
  } finally {
    context.mock.restoreAll();
    syncBuiltinESMExports();
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("foreign physical links retain their raw bytes and remain strict refusals", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-foreign-evidence-"));
  try {
    const { nodes, children } = fixture(temporary);
    symlinkSync("../../outside", join(temporary, "jellyfin-vue/untracked"));
    let observed;
    assert.throws(
      () => collectNativePhysical(temporary, nodes, children, (raw) => (observed = raw)),
      /Foreign physical canonical symlink/,
    );
    const physical = observed.physical.find((row) => row.path === "jellyfin-vue/untracked");
    assert.equal(physical.targetBase64, Buffer.from("../../outside").toString("base64"));
    assert.equal(physical.mode & 0o170000, 0o120000);
    assert.deepEqual(observed.metadata, []);
    assert.deepEqual(observed.files, []);
    assert.deepEqual(observed.rejected, []);
    assert.equal(observed.successfulFollows, 0);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("unchanged original Rust, physical bytes and independent logical Git walk agree at the actual kernel boundary", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-native-walk-"));
  const root = join(temporary, "shape");
  mkdirSync(root);
  try {
    const { nodes, children } = fixture(root);
    const source = readFileSync("tests/tooling/fixtures/canonical_corpus_collector_original.rs");
    const { body, harness } = collectorHarness(source);
    assert.equal(Buffer.byteLength(body), 778, "The original function body is not rewritten");
    writeFileSync(join(temporary, "original.rs"), harness);
    const binary = join(temporary, "original");
    execFileSync("rustc", ["--edition=2024", join(temporary, "original.rs"), "-o", binary]);
    const native = parseNativeVector(execFileSync(binary, [root]));
    const physical = collectNativePhysical(root, nodes, children);
    const committed = walkCommittedGraph(nodes, children, physical.boundary).files;
    assert.deepEqual(
      native,
      physical.files.map(([path]) => path),
    );
    assert.deepEqual(
      physical.files.map(([path, , , blob]) => [path, blob]),
      committed,
    );
    assert(native.includes("jellyfin-vue/src/Z.vue"));
    assert(native.includes("jellyfin-vue/copy/Z.vue"), "Physical aliases are not deduplicated");
    assert(!native.some((path) => path.endsWith("/.vue") || path.includes("/node_modules/")));
    const atRoot = native.filter((path) => /^jellyfin-vue\/src\/[^/]+$/.test(path));
    assert.deepEqual(atRoot, [
      "jellyfin-vue/src/Z.vue",
      "jellyfin-vue/src/\ue000.vue",
      "jellyfin-vue/src/𐀀.vue",
    ]);
    assert(
      pathOrder("\ue000.vue", "𐀀.vue") < 0,
      "Use original UTF8/OsStr order, not UTF16 sorting",
    );
    assert(physical.boundary.firstRejected > 1);
    const cropped = { ...physical.boundary, firstRejected: physical.boundary.firstRejected - 1 };
    assert.throws(() => walkCommittedGraph(nodes, children, cropped), /boundary/);
    const forgedPaths = structuredClone(physical.boundary);
    forgedPaths.rejected.pop();
    assert.throws(() => walkCommittedGraph(nodes, children, forgedPaths), /boundary/);
    const wrongCode = { ...physical.boundary, code: "ENAMETOOLONG" };
    assert.throws(() => walkCommittedGraph(nodes, children, wrongCode), /Unqualified/);
    const wrongPin = structuredClone(nodes);
    wrongPin.get(cycleOwner.path).revision = "a".repeat(40);
    assert.throws(() => validateLogicalGraph(wrongPin, children), /directory cycle/);
    const wrongBlob = structuredClone(nodes);
    wrongBlob.get(cycleOwner.path).id = "b".repeat(40);
    assert.throws(() => validateLogicalGraph(wrongBlob, children), /directory cycle/);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("current fail-closed Rust collector qualifies finite routes while retaining complete original kernel paths and bytes", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-current-native-walk-"));
  const root = join(temporary, "shape");
  mkdirSync(root);
  try {
    const { nodes, children } = fixture(root);
    const versions = [
      readFileSync("tests/tooling/fixtures/canonical_corpus_collector_original.rs"),
      readFileSync("tests/davinci_test_support/src/corpus.rs"),
    ];
    const vectors = versions.map((source, index) => {
      const { harness } = collectorHarness(source);
      const input = join(temporary, `collector-${index}.rs`);
      const binary = join(temporary, `collector-${index}`);
      writeFileSync(input, harness);
      execFileSync("rustc", ["--edition=2024", input, "-o", binary]);
      return execFileSync(binary, [root]);
    });
    const physical = collectNativePhysical(root, nodes, children);
    assert.deepEqual(
      vectors[0],
      Buffer.from(physical.files.map(([path]) => `${path}\0`).join("")),
      "Every original ordered path and NUL is retained in the kernel observation",
    );
    const finite = collectFinitePhysical(root, nodes, children, physical.boundary);
    assert.deepEqual(
      parseNativeVector(vectors[1]),
      finite.map(([path]) => path),
    );
    assert.deepEqual(
      physical.files.map(([path, , , blob]) => [path, blob]),
      walkCommittedGraph(nodes, children, physical.boundary).files,
    );
    const qualified = finiteCommittedInventory(nodes, children, physical.boundary);
    assert.deepEqual(
      validateFiniteNativeVector(vectors[1], nodes, children, physical.boundary),
      finite.map(([path]) => path),
    );
    for (const paths of [
      finite.slice(1).map(([path]) => path),
      finite.map(([path]) => path).reverse(),
      [...finite.map(([path]) => path), "jellyfin-vue/uncommitted.vue"],
    ])
      assert.throws(
        () =>
          validateFiniteNativeVector(
            Buffer.from(paths.map((path) => `${path}\0`).join("")),
            nodes,
            children,
            physical.boundary,
          ),
        /committed routes/,
      );
    assert.deepEqual(
      finite.map(([path, , , blob]) => [path, blob]),
      qualified.files,
    );
    assert(qualified.cut.length > 0);
    assert(physical.files.length > finite.length);
    const originalBytes = new Map(
      physical.files.map((row) => [resolveLogical(nodes, row[0]).path, row.slice(1)]),
    );
    const finiteBytes = new Map(
      finite.map((row) => [resolveLogical(nodes, row[0]).path, row.slice(1)]),
    );
    assert.deepEqual(
      finiteBytes,
      originalBytes,
      "All original physical sources retain whole bytes and Git blobs",
    );
    for (const path of [
      "jellyfin-vue/copy/Z.vue",
      "jellyfin-vue/packaging/deb/root/copy/Z.vue",
      "jellyfin-vue/src/Z.vue",
    ])
      assert(
        parseNativeVector(vectors[1]).includes(path),
        `Independent logical alias retained: ${path}`,
      );
    const missing = join(temporary, "missing corpus root");
    assert.throws(
      () =>
        execFileSync(join(temporary, "collector-1"), [missing], {
          stdio: ["ignore", "pipe", "pipe"],
        }),
      (error) => {
        assert.equal(error.status, 101);
        assert.deepEqual(error.stdout, Buffer.alloc(0));
        const stderr = error.stderr.toString("utf8");
        assert(stderr.includes(`corpus IO failed: read directory ${missing}:`));
        assert(stderr.includes("kind=NotFound"));
        assert(/raw_os_error=Some\(\d+\)/.test(stderr));
        return true;
      },
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("native collector includes exact required IO helpers and refuses omitted or duplicated definitions", () => {
  const source = readFileSync("tests/davinci_test_support/src/corpus.rs", "utf8");
  const { harness } = collectorHarness(Buffer.from(source));
  for (const name of ["fail_corpus_io", "corpus_entry_path", "collect_vue_edges"]) {
    const definition = source.match(new RegExp(`^fn ${name}\\([\\s\\S]*?\\n\\}\\n`, "m"))?.[0];
    assert(definition, `Original helper ${name} is present`);
    assert(harness.toString("utf8").includes(definition));
    assert.throws(() => collectorHarness(Buffer.from(source.replace(definition, ""))), /unique/);
    assert.throws(() => collectorHarness(Buffer.from(source + definition)), /unique/);
  }
});

await test("foreign links, arbitrary ancestor/self cycles and original unreadable inputs remain refusals", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-native-refusal-"));
  try {
    const { nodes, children, add } = fixture(temporary);
    add("jellyfin-vue/foreign", "120000", Buffer.from("../../outside"));
    assert.throws(() => validateLogicalGraph(nodes, children), /Foreign|Missing/);
    nodes.delete("jellyfin-vue/foreign");
    children.get("jellyfin-vue").delete("foreign");
    rmSync(join(temporary, "jellyfin-vue/foreign"));
    add("jellyfin-vue/loop", "120000", Buffer.from("loop/more"));
    assert.throws(() => resolveLogical(nodes, "jellyfin-vue/loop"), /symlink cycle/);
    nodes.get("jellyfin-vue/loop").target = ".";
    assert.throws(() => validateLogicalGraph(nodes, children), /directory cycle/);
    nodes.delete("jellyfin-vue/loop");
    children.get("jellyfin-vue").delete("loop");
    rmSync(join(temporary, "jellyfin-vue/loop"));
    symlinkSync("../../outside", join(temporary, "jellyfin-vue/untracked"));
    assert.throws(() => collectNativePhysical(temporary, nodes, children), /Foreign physical/);
    rmSync(join(temporary, "jellyfin-vue/untracked"));
    const boundary = collectNativePhysical(temporary, nodes, children).boundary;
    rmSync(join(temporary, "jellyfin-vue/src/Z.vue"));
    assert.throws(
      () => collectNativePhysical(temporary, nodes, children),
      /ENOENT|differs|boundary/,
    );
    assert.throws(
      () => collectFinitePhysical(temporary, nodes, children, boundary),
      /whole vector differs/,
    );
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

await test("native whole vectors and original-body extraction reject malformed encodings, paths and truncation", () => {
  for (const bytes of [
    Buffer.from("one.vue"),
    Buffer.from("../one.vue\0"),
    Buffer.from("/one.vue\0"),
    Buffer.from("one.vue\0one.vue\0"),
    Buffer.from([255, 0]),
  ])
    assert.throws(() => parseNativeVector(bytes));
  const source = readFileSync("tests/davinci_test_support/src/corpus.rs");
  const { body } = collectorHarness(source);
  assert.throws(() => collectorHarness(Buffer.from("missing source")), /unique/);
  assert.throws(() => collectorHarness(Buffer.concat([source, Buffer.from(body)])), /unique/);
});
