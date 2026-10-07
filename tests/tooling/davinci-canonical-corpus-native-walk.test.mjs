import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, posix } from "node:path";
import { test } from "node:test";
import { gitObjectId } from "../../tools/support/compat/github/canonical-corpus-inventory.mjs";
import {
  cycleOwner,
  pathOrder,
  resolveLogical,
  validateLogicalGraph,
  walkCommittedGraph,
} from "../../tools/support/compat/github/canonical-corpus-logical-walk.mjs";
import {
  collectorHarness,
  collectNativePhysical,
  parseNativeVector,
  validateNativeSource,
} from "../../tools/support/compat/github/canonical-corpus-native-walk.mjs";

// Authored small controls exercise the unchanged real Rust function, actual
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

await test("unchanged original Rust, physical bytes and independent logical Git walk agree at the actual kernel boundary", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-native-walk-"));
  const root = join(temporary, "shape");
  mkdirSync(root);
  try {
    const { nodes, children } = fixture(root);
    const source = readFileSync("tests/davinci_test_support/src/corpus.rs");
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
    rmSync(join(temporary, "jellyfin-vue/src/Z.vue"));
    assert.throws(
      () => collectNativePhysical(temporary, nodes, children),
      /ENOENT|differs|boundary/,
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

await test("native archive source/head/toolchain/binary and entire raw-vector custody refuses replacements", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-native-archive-"));
  const root = join(temporary, "inputs");
  mkdirSync(root);
  writeFileSync(join(root, "Original.vue"), "<template>whole original</template>\r\n");
  const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
  const identity = {
    sha: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    runId: 12,
    attempt: 1,
  };
  try {
    const source = readFileSync("tests/davinci_test_support/src/corpus.rs");
    const { body, harness } = collectorHarness(source);
    writeFileSync(join(temporary, "native-collector-source.rs"), source);
    writeFileSync(join(temporary, "native-collector.rs"), harness);
    execFileSync("rustc", [
      "--edition=2024",
      join(temporary, "native-collector.rs"),
      "-o",
      join(temporary, "native-collector"),
    ]);
    const toolchain = execFileSync("rustc", ["-vV"]);
    const vector = execFileSync(join(temporary, "native-collector"), [root]);
    writeFileSync(join(temporary, "native-rustc.txt"), toolchain);
    writeFileSync(join(temporary, "native-vector.nul"), vector);
    const receipt = {
      schema: "vize.canonical-native-walk",
      version: 1,
      ...identity,
      sourceSha256: hash(source),
      bodySha256: hash(body),
      harnessSha256: hash(harness),
      binarySha256: hash(readFileSync(join(temporary, "native-collector"))),
      toolchainSha256: hash(toolchain),
      vectorSha256: hash(vector),
      files: 1,
    };
    const receiptPath = join(temporary, "native-walk.json");
    writeFileSync(receiptPath, JSON.stringify(receipt));
    assert.deepEqual(validateNativeSource(temporary, identity).paths, ["Original.vue"]);
    for (const name of [
      "native-collector-source.rs",
      "native-collector.rs",
      "native-collector",
      "native-rustc.txt",
      "native-vector.nul",
    ]) {
      const path = join(temporary, name);
      const original = readFileSync(path);
      writeFileSync(path, Buffer.concat([original, Buffer.from("changed")]));
      assert.throws(() => validateNativeSource(temporary, identity), `Changed ${name}`);
      writeFileSync(path, original);
    }
    for (const key of [
      "sha",
      "tree",
      "runId",
      "attempt",
      "sourceSha256",
      "bodySha256",
      "harnessSha256",
      "binarySha256",
      "toolchainSha256",
      "vectorSha256",
      "files",
    ]) {
      writeFileSync(receiptPath, JSON.stringify({ ...receipt, [key]: "foreign" }));
      assert.throws(() => validateNativeSource(temporary, identity), `Foreign ${key}`);
    }
    writeFileSync(receiptPath, JSON.stringify(receipt));
    assert.deepEqual(validateNativeSource(temporary, identity).paths, ["Original.vue"]);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
