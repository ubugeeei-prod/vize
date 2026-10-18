import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, rmSync, statSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, posix } from "node:path";
import { test } from "node:test";
import {
  cycleOwner,
  kernelPrefixWitness,
  qualifyKernelBoundary,
  resolveLogical,
  walkCommittedGraph,
  type CorpusNodes,
  type CorpusChildren,
  type CorpusNode,
  type VueFile,
} from "../../tools/support/compat/github/canonical-corpus-logical-walk.ts";
import {
  collectNativePhysical,
  parseNativeVector,
} from "../../tools/support/compat/github/canonical-corpus-native-walk.ts";

const gitBlob = (bytes: Buffer) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
const sources = [
  ["Z.vue", Buffer.from("<template>ASCII\r\n</template>\r\n")],
  ["\ue000.vue", Buffer.from("<template>private-use</template>\n")],
  ["𐀀.vue", Buffer.from("<template>astral</template>\n")],
] satisfies Array<[string, Buffer]>;

function fixture(root: string) {
  const nodes: CorpusNodes = new Map([["", { path: "", mode: "40000" }]]);
  const children: CorpusChildren = new Map([["", new Set()]]);
  function add(path: string, mode: string, bytes = Buffer.alloc(0)) {
    const parent = posix.dirname(path) === "." ? "" : posix.dirname(path);
    if (!nodes.has(parent)) add(parent, "40000");
    const node: CorpusNode = {
      path,
      mode,
      repository: cycleOwner.repository,
      revision: cycleOwner.revision,
    };
    if (mode === "40000") {
      children.set(path, new Set());
      mkdirSync(join(root, path), { recursive: true });
    } else {
      node.id = gitBlob(bytes);
      if (mode === "120000") {
        node.target = bytes.toString("utf8");
        symlinkSync(node.target, join(root, path));
      } else writeFileSync(join(root, path), bytes);
    }
    nodes.set(path, node);
    children.get(parent)!.add(posix.basename(path));
  }
  for (const [name, bytes] of sources) add(`jellyfin-vue/src/${name}`, "100644", bytes);
  add("jellyfin-vue/src/.vue", "100644", Buffer.from("not a Rust extension match\n"));
  add("jellyfin-vue/node_modules/Excluded.vue", "100644", Buffer.from("excluded\n"));
  add(cycleOwner.path, "120000", Buffer.from(cycleOwner.target));
  add("jellyfin-vue/a-chain", "120000", Buffer.from("b-chain"));
  add("jellyfin-vue/b-chain", "120000", Buffer.from("src"));
  add("jellyfin-vue/copy", "120000", Buffer.from("src"));
  return { nodes, children };
}

// Independent layout oracle: enumerate every original logical alias in native
// byte order, including every repeated ancestor prefix accepted by this kernel.
function expectedFiles(firstRefused: number): VueFile[] {
  const files: VueFile[] = [];
  function source(directory: string) {
    for (const [name, bytes] of sources)
      files.push([
        `${directory}/${name}`,
        createHash("sha256").update(bytes).digest("hex"),
        bytes.length,
        gitBlob(bytes),
      ]);
  }
  function visit(directory: string, depth: number) {
    for (const [name, follows] of [
      ["a-chain", 2],
      ["b-chain", 1],
      ["copy", 1],
    ] satisfies Array<[string, number]>)
      if (depth + follows < firstRefused) source(`${directory}/${name}`);
    if (depth + 1 < firstRefused) visit(`${directory}/packaging/deb/root`, depth + 1);
    source(`${directory}/src`);
  }
  visit("jellyfin-vue", 0);
  return files;
}

await test("actual kernel prefix preserves full rejected finite-alias provenance and every whole Vue byte", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-kernel-prefix-"));
  try {
    const { nodes, children } = fixture(root);
    const physical = collectNativePhysical(root, nodes, children);
    const witnesses = physical.boundary.rejected.flatMap((row) => {
      const parent = posix.dirname(row.path);
      const witness = kernelPrefixWitness(nodes, row.path, parent);
      if (!witness) return [];
      assert.equal(statSync(join(root, parent)).isDirectory(), true);
      assert.throws(() => statSync(join(root, row.path)), { code: "ELOOP" });
      assert.equal(witness.preceding.followed + 1, witness.followed);
      return [witness];
    });
    assert.equal(witnesses.length, 1);
    const firstRefused = witnesses[0].followed;
    assert.equal(physical.boundary.firstRejected, firstRefused);
    assert.deepEqual(
      [...new Set(physical.boundary.rejected.map((row) => row.followed))].sort((a, b) => a - b),
      [firstRefused, firstRefused + 1],
    );
    assert.deepEqual(physical.files, expectedFiles(firstRefused));
    assert.deepEqual(
      physical.boundary.rejected.map((row) => ({
        ...row,
        followed: resolveLogical(nodes, row.path).followed,
      })),
      physical.boundary.rejected,
    );
    const vector = Buffer.from(physical.files.map(([path]) => `${path}\0`).join(""));
    assert.deepEqual(
      vector,
      Buffer.from(
        expectedFiles(firstRefused)
          .map(([path]) => `${path}\0`)
          .join(""),
      ),
    );
    assert.deepEqual(
      parseNativeVector(vector),
      physical.files.map(([path]) => path),
    );
    const committed = walkCommittedGraph(nodes, children, physical.boundary);
    assert.deepEqual(committed.rejected, physical.boundary.rejected);
    assert.deepEqual(
      committed.files,
      physical.files.map(([path, , , blob]) => [path, blob]),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("a logical full-path count cannot replace the actual owned prefix witness", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-kernel-prefix-refusal-"));
  try {
    const { nodes, children } = fixture(root);
    const physical = collectNativePhysical(root, nodes, children);
    const witnesses = physical.boundary.rejected.flatMap(
      (row) => kernelPrefixWitness(nodes, row.path, posix.dirname(row.path)) ?? [],
    );
    const successful = witnesses[0].preceding.followed;
    assert.throws(
      () => qualifyKernelBoundary(nodes, physical.boundary.rejected, successful, []),
      /prefix witness/,
    );
    assert.throws(() => qualifyKernelBoundary(nodes, [], successful, witnesses), /actual native/);
    const forged = structuredClone(witnesses);
    forged[0].preceding.followed += 1;
    assert.throws(
      () => qualifyKernelBoundary(nodes, physical.boundary.rejected, successful, forged),
      /Foreign kernel prefix/,
    );
    assert.throws(
      () => qualifyKernelBoundary(nodes, physical.boundary.rejected, successful - 1, witnesses),
      /preceding success/,
    );
    const missing = structuredClone(physical.boundary);
    missing.rejected.pop();
    assert.throws(() => walkCommittedGraph(nodes, children, missing), /boundary paths/);
    const truncated = structuredClone(physical.boundary);
    truncated.rejected = truncated.rejected.map((row) => ({
      ...row,
      followed: truncated.firstRejected,
    }));
    assert.throws(() => walkCommittedGraph(nodes, children, truncated), /boundary paths/);
    const foreign = structuredClone(nodes);
    foreign.get(cycleOwner.path)!.id = "b".repeat(40);
    assert.equal(
      kernelPrefixWitness(foreign, witnesses[0].path, witnesses[0].preceding.path),
      undefined,
    );
    assert.throws(
      () => qualifyKernelBoundary(foreign, physical.boundary.rejected, successful, witnesses),
      /Foreign kernel prefix/,
    );
    assert.throws(
      () => walkCommittedGraph(foreign, children, physical.boundary),
      /directory cycle/,
    );
    const ordinary = structuredClone(nodes);
    ordinary.get(cycleOwner.path)!.mode = "100644";
    assert.equal(
      kernelPrefixWitness(ordinary, cycleOwner.path, posix.dirname(cycleOwner.path)),
      undefined,
    );
    assert.throws(
      () => qualifyKernelBoundary(ordinary, physical.boundary.rejected, successful, witnesses),
      { message: "Invalid committed symlink traversal" },
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("logical traversal preserves complete finite remainders beyond an authored synthetic boundary", () => {
  // This is a logical law, not an assertion about any platform's kernel limit.
  const nodes: CorpusNodes = new Map([["", { path: "", mode: "40000" }]]);
  const children: CorpusChildren = new Map([["", new Set()]]);
  function add(path: string, mode: string, target?: string) {
    const parent = posix.dirname(path) === "." ? "" : posix.dirname(path);
    if (!nodes.has(parent)) add(parent, "40000");
    const node: CorpusNode = {
      path,
      mode,
      repository: cycleOwner.repository,
      revision: cycleOwner.revision,
    };
    if (mode === "40000") children.set(path, new Set());
    else if (target) Object.assign(node, { target, id: gitBlob(Buffer.from(target)) });
    else node.id = gitBlob(sources[0][1]);
    nodes.set(path, node);
    children.get(parent)!.add(posix.basename(path));
  }
  add("jellyfin-vue/src/Z.vue", "100644");
  add(cycleOwner.path, "120000", cycleOwner.target);
  add("jellyfin-vue/a-chain", "120000", "b-chain");
  add("jellyfin-vue/b-chain", "120000", "src");
  const at = (depth: number, suffix: string) =>
    `jellyfin-vue/${"packaging/deb/root/".repeat(depth)}${suffix}`;
  const boundary = {
    code: "ELOOP",
    firstRejected: 4,
    rejected: [
      { path: at(2, "a-chain"), code: "ELOOP", followed: 4 },
      { path: at(3, "a-chain"), code: "ELOOP", followed: 5 },
      { path: at(3, "b-chain"), code: "ELOOP", followed: 4 },
      { path: at(3, "packaging/deb/root"), code: "ELOOP", followed: 4 },
    ],
  };
  const observed = walkCommittedGraph(nodes, children, boundary);
  assert.deepEqual(observed.rejected, boundary.rejected);
  assert.deepEqual(
    observed.files,
    [
      at(0, "a-chain/Z.vue"),
      at(0, "b-chain/Z.vue"),
      at(1, "a-chain/Z.vue"),
      at(1, "b-chain/Z.vue"),
      at(2, "b-chain/Z.vue"),
      at(3, "src/Z.vue"),
      at(2, "src/Z.vue"),
      at(1, "src/Z.vue"),
      at(0, "src/Z.vue"),
    ].map((path) => [path, gitBlob(sources[0][1])]),
  );
  const omittedRemainder = structuredClone(boundary);
  omittedRemainder.rejected[1].followed = 4;
  assert.throws(() => walkCommittedGraph(nodes, children, omittedRemainder), /boundary paths/);
});
