import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertDeclarationCustody,
  captureDeclarationGraph,
} from "../../../tools/support/release/computed_event_custody.ts";

const typeImport = (specifier: string, mode = "import") =>
  `import type {} from ${JSON.stringify(specifier)} with { "resolution-mode": "${mode}" };\n`;
function fixture() {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-computed-module-"));
  const root = path.join(directory, "vize");
  mkdirSync(path.join(root, "generated"), { recursive: true });
  for (const name of [
    "virtual-ts.d.ts",
    "virtual-ts.d.mts",
    "virtual-ts.d.cts",
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
    "generated/content-mapper-types.d.ts",
    "virtual-ts-globals.d.ts",
    "virtual-ts-patterns.d.ts",
  ])
    writeFileSync(path.join(root, name), "export {};\n");
  for (const name of ["virtual-ts.d.ts", "virtual-ts.d.mts", "virtual-ts.d.cts"])
    writeFileSync(
      path.join(root, name),
      typeImport("./virtual-ts-globals.js") +
        typeImport("./virtual-ts-patterns.js") +
        'type External = import("vue").Unknown;\n',
    );
  return { directory, root: realpathSync(root) };
}

test("module declaration custody resolves the six V-shaped type imports and retains mode attributes", () => {
  const f = fixture();
  try {
    // Never load the runtime module as a substitute for its declaration.
    writeFileSync(
      path.join(f.root, "virtual-ts-globals.js"),
      'throw new Error("must not execute");\n',
    );
    const graph = captureDeclarationGraph(f.root);
    assert.equal(graph.files.length, 9);
    assert.equal(graph.edges.length, 6);
    for (const edge of graph.edges) {
      assert.equal(edge.kind, "type-import");
      assert.deepEqual(edge.attributes, { "resolution-mode": "import" });
      assert(edge.target.endsWith(".d.ts"));
      assert.equal(
        graph.files.some((file) => file.path === edge.target),
        true,
      );
    }
    assertDeclarationCustody(f.root, graph.files, graph.edges);
    graph.edges[0]!.attributes["resolution-mode"] = "require";
    assert.throws(
      () => assertDeclarationCustody(f.root, graph.files, graph.edges),
      /module edges changed/u,
    );
  } finally {
    rmSync(f.directory, { recursive: true });
  }
});

test("module declaration substitution preserves mjs/cjs kinds and finite re-export cycles", () => {
  const f = fixture();
  try {
    mkdirSync(path.join(f.root, "nested"));
    writeFileSync(
      path.join(f.root, "nested/leaf.d.mts"),
      'export type * from "../virtual-ts-globals.js" with { "resolution-mode": "import" };\n',
    );
    writeFileSync(path.join(f.root, "nested/common.d.cts"), "export {};\n");
    writeFileSync(
      path.join(f.root, "virtual-ts-globals.d.ts"),
      typeImport("./nested/leaf.mjs") + typeImport("./nested/common.cjs", "require"),
    );
    const graph = captureDeclarationGraph(f.root);
    assert.equal(graph.files.length, 11);
    assert(
      graph.edges.some(
        (edge) =>
          edge.target.endsWith("leaf.d.mts") && edge.attributes["resolution-mode"] === "import",
      ),
    );
    assert(
      graph.edges.some(
        (edge) =>
          edge.target.endsWith("common.d.cts") && edge.attributes["resolution-mode"] === "require",
      ),
    );
    assert(graph.edges.some((edge) => edge.kind === "type-export"));
  } finally {
    rmSync(f.directory, { recursive: true });
  }
});

test("module declaration custody follows type queries while ignoring comment/string lookalikes and bare Vue", () => {
  const f = fixture();
  try {
    writeFileSync(path.join(f.root, "leaf.d.ts"), "export {};\n");
    writeFileSync(
      path.join(f.root, "virtual-ts-globals.d.ts"),
      [
        '// import type {} from "../not-real.js" with { "resolution-mode": "import" };',
        'type Text = "import(\\\"../not-real.js\\\")";',
        'import type { Ref } from "vue";',
        'type Bare = import("vue").Unknown;',
        'type Local = `${import("./leaf.js", { with: { "resolution-mode": "import" } }).Value}`;',
        "export {};\n",
      ].join("\n"),
    );
    const graph = captureDeclarationGraph(f.root);
    assert.equal(graph.files.length, 10);
    const edge = graph.edges.find((edge) => edge.kind === "import-type");
    assert(edge);
    assert(edge.target.endsWith("leaf.d.ts"));
    assert.deepEqual(edge.attributes, { "resolution-mode": "import" });
  } finally {
    rmSync(f.directory, { recursive: true });
  }
});

test("module declaration imports reject malformed modes, missing declarations and escaped targets", () => {
  const f = fixture();
  try {
    writeFileSync(path.join(f.directory, "outside.d.ts"), "export {};\n");
    writeFileSync(path.join(f.root, "missing.js"), 'throw new Error("not a declaration");\n');
    for (const [source, error] of [
      [typeImport("../outside.js"), /escapes installed package/u],
      [
        'import type {} from "\\u002e\\u002e/outside.js" with { "resolution-mode": "import" };',
        /escapes installed package/u,
      ],
      [typeImport("file:///outside.js"), /must stay local/u],
      [typeImport("./missing.js"), /ENOENT/u],
      [typeImport("./virtual-ts-globals.js", "sideways"), /invalid declaration resolution-mode/u],
      [
        'import type {} from "./virtual-ts-globals.js" with { "resolution-mode": import };',
        /must be a string/u,
      ],
      [
        'import type {} from "./virtual-ts-globals.js" with { "resolution-mode": "import", "resolution-mode": "require" };',
        /duplicate/u,
      ],
      [
        'import type {} from "./virtual-ts-globals.js" with { "resolution-mode": "import";',
        /malformed declaration import/u,
      ],
    ] as const) {
      writeFileSync(path.join(f.root, "virtual-ts.d.ts"), source);
      assert.throws(() => captureDeclarationGraph(f.root), error);
    }
  } finally {
    rmSync(f.directory, { recursive: true });
  }
});
