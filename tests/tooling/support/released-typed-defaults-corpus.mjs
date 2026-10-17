import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  apiRoot,
  gitBlob,
  sha256,
} from "../../../tools/support/release/typed_defaults/custody.mjs";
import {
  originals,
  originalSourcePaths,
  typedDirectory,
} from "../../../tools/support/release/typed_defaults/fixtures.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));

/** Inert provider uses the original authored file bytes, never CLI observations. */
export function corpusFixture() {
  const head = "d".repeat(40),
    treeSha = "b".repeat(40),
    rows = new Map();
  const tree = { sha: treeSha, truncated: false, tree: [] };
  rows.set(`${apiRoot}/git/trees/${treeSha}?recursive=1`, tree);
  function add(name, bytes) {
    const sha = gitBlob(bytes);
    tree.tree.push({ path: name, type: "blob", mode: "100644", sha });
    rows.set(`${apiRoot}/git/blobs/${sha}`, {
      sha,
      encoding: "base64",
      size: bytes.length,
      content: bytes.toString("base64"),
    });
  }
  for (const [name, hash] of Object.entries(originals)) {
    const sourcePath = originalSourcePaths[name] ?? name;
    const bytes = fs.readFileSync(path.join(root, sourcePath));
    assert.equal(sha256(bytes), hash, `authored control bytes changed: ${name}`);
    add(sourcePath, bytes);
    if (!name.endsWith("cases.json")) continue;
    for (const entry of JSON.parse(bytes).cases) {
      const filename = `${path.dirname(name)}/${entry.filename}.fixture`;
      add(filename, fs.readFileSync(path.join(root, filename)));
    }
  }
  assert(tree.tree.some((entry) => entry.path === `${typedDirectory}/DrawerContent.vue.fixture`));
  return {
    head,
    treeSha,
    rows,
    tree,
    read: async (url) => {
      assert(rows.has(url), `unplanned original source read: ${url}`);
      return structuredClone(rows.get(url));
    },
  };
}
