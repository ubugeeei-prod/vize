/** Current source SDK only. Historical/public declaration inventories remain separate. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import {
  captureDeclarationGraph,
  inside,
  type DeclarationGraph,
} from "../../release/computed_event_custody.ts";

const privateLeaf = "virtual-ts-emits.d.ts";
const virtualEntries = ["virtual-ts.d.ts", "virtual-ts.d.mts", "virtual-ts.d.cts"];

function exportTargets(value: unknown): string[] {
  if (typeof value === "string") return [value];
  if (value === null) return [];
  if (Array.isArray(value)) return value.flatMap((item: unknown) => exportTargets(item));
  return Object.values(asRecord(value)).flatMap((item) => exportTargets(item));
}

function exposesPrivateLeaf(target: string) {
  const normalized = target.replace(/^\.\//u, "");
  const pattern = normalized
    .split("*")
    .map((part) => part.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&"))
    .join(".*");
  const matcher = new RegExp(`^${pattern}$`, "u");
  return [privateLeaf, "virtual-ts-emits.js", "virtual-ts-emits"].some((file) =>
    matcher.test(file),
  );
}

/** Authenticate advertised files, every local edge, and the private emit leaf's real consumers. */
export function captureCurrentSdkDeclarations(directory: string) {
  const root = fs.realpathSync(directory);
  const manifest = asRecord(parseJson(fs.readFileSync(path.join(root, "package.json"), "utf8")));
  const files = asArray(manifest.files)
    .map((file) => asString(file))
    .filter((file) => /\.d\.(?:ts|mts|cts)$/u.test(file));
  assert.equal(new Set(files).size, files.length, "unique current SDK declaration inventory");
  for (const file of files) {
    assert.ok(!path.isAbsolute(file) && !path.win32.isAbsolute(file));
    assert.ok(!file.includes("\\") && !file.split("/").includes(".."));
    assert.ok(inside(root, path.resolve(root, file)));
  }
  assert.ok(files.includes(privateLeaf), "the private emit dependency must be shipped");
  assert.ok(
    !exportTargets(manifest.exports).some((target) => exposesPrivateLeaf(target)),
    "the private emit leaf cannot become a package subpath export",
  );
  const exports = asRecord(manifest.exports);
  for (const subpath of ["./virtual-ts", "./virtual-ts-checker", "./content-mapper-helpers"])
    for (const target of exportTargets(exports[subpath]))
      assert.ok(
        files.includes(target.replace(/^\.\//u, "")),
        `${subpath} selects a shipped declaration`,
      );
  const graph = captureDeclarationGraph(root, files);
  assert.deepEqual(
    graph.files.map((file) => path.relative(root, file.path).replaceAll("\\", "/")).sort(),
    [...files].sort(),
    "every transitive declaration must be physically shipped by this SDK",
  );
  const leaf = fs.realpathSync(path.join(root, privateLeaf));
  for (const file of virtualEntries) {
    const edges = graph.edges.filter(
      (edge) => edge.from === path.join(root, file) && edge.target === leaf,
    );
    assert.equal(edges.length, 1, `${file} keeps its private factory dependency`);
    assert.equal(edges[0]!.kind, "type-import");
    assert.equal(edges[0]!.specifier, "./virtual-ts-emits.js");
    assert.deepEqual(edges[0]!.attributes, { "resolution-mode": "import" });
  }
  const base = graph.edges.filter((edge) => edge.from === leaf);
  assert.equal(base.length, 1, "private factories keep their base-helper dependency");
  assert.equal(base[0]!.target, path.join(root, "virtual-ts.d.ts"));
  assert.equal(base[0]!.kind, "type-import");
  assert.deepEqual(base[0]!.attributes, { "resolution-mode": "import" });
  const mapperGraph = captureDeclarationGraph(root);
  for (const image of mapperGraph.files)
    assert.ok(files.includes(path.relative(root, image.path).replaceAll("\\", "/")));
  return { root, files, graph, mapperGraph };
}

export function relativeDeclarationGraph(graph: DeclarationGraph, root: string) {
  return {
    files: graph.files.map((file) => ({ ...file, path: path.relative(root, file.path) })),
    edges: graph.edges.map((edge) => ({
      ...edge,
      from: path.relative(root, edge.from),
      target: path.relative(root, edge.target),
    })),
  };
}
