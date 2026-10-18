import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { firstLocation, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root, testOutputRoot } from "./support/lsp/paths.ts";
import type { LspRange, ServerCapabilities } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";

const source = `<script setup lang="ts">
interface Formatter {
  format(value: string): string
}

class LabelFormatter implements Formatter {
  format(value: string): string {
    return value.toUpperCase()
  }
}

const formatter: Formatter = new LabelFormatter()
const message = "hello"
</script>

<template>
  <span>{{ formatter.format(message) }}</span>
</template>
`;

const tsxSource = `interface Formatter {
  format(value: string): string
}

class LabelFormatter implements Formatter {
  format(value: string): string {
    return value.toUpperCase()
  }
}

const formatter: Formatter = new LabelFormatter()

export const view = <span>{formatter.format("hello")}</span>
`;

type ImplementationCapabilities = ServerCapabilities & {
  implementationProvider?: unknown;
};

test("vize lsp maps textDocument/implementation from authored SFC interfaces", async () => {
  const testRootDir = path.join(testOutputRoot, "lsp-implementation-capability");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
        },
        include: ["**/*"],
      },
      null,
      2,
    ),
  );
  const filePath = path.join(workspaceDir, "Widget.vue");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: ImplementationCapabilities };
    assert.equal(init.capabilities?.implementationProvider, true);

    fs.writeFileSync(filePath, source, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const position = offsetToPosition(source, source.indexOf("format(value: string): string"));
    const implementation = await session.request("textDocument/implementation", {
      textDocument: { uri },
      position,
    });
    const location = firstLocation(implementation as never);
    assert.equal(location.uri, uri);
    assert.deepEqual(
      location.range.start,
      offsetToPosition(source, source.indexOf("format(value: string): string {")),
    );
    assert.ok(!location.uri.endsWith(".vue.ts"), JSON.stringify(implementation));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

test("vize lsp maps textDocument/implementation from authored TSX interfaces", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "TypeScript 7/Corsa runtime for TSX implementation navigation",
    "TypeScript 7/Corsa runtime not found; skipping TSX implementation navigation test",
  );
  if (corsaPath == null) return;

  const testRootDir = path.join(testOutputRoot, "lsp-tsx-implementation-capability");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const sourceDir = path.join(workspaceDir, "src");
  fs.mkdirSync(sourceDir, { recursive: true });
  writeVueShim(sourceDir);
  fs.writeFileSync(
    path.join(workspaceDir, "vize.config.json"),
    JSON.stringify({
      lsp: { lint: false, typecheck: true },
      typeChecker: { corsaPath, jsxTypecheck: true },
    }),
    "utf8",
  );
  fs.writeFileSync(
    path.join(workspaceDir, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          jsx: "preserve",
          jsxImportSource: "vue",
          module: "ESNext",
          moduleResolution: "bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/**/*"],
      },
      null,
      2,
    ),
  );
  const filePath = path.join(sourceDir, "Widget.tsx");
  const uri = pathToFileURL(filePath).href;
  const session = new LspSession();

  try {
    const init = (await session.initialize(workspaceDir, {
      editor: true,
      lint: false,
      typecheck: true,
    })) as { capabilities?: ImplementationCapabilities };
    assert.equal(init.capabilities?.implementationProvider, true);

    fs.writeFileSync(filePath, tsxSource, "utf8");
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "typescriptreact", version: 1, text: tsxSource },
    });
    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri),
      60_000,
    );

    const position = offsetToPosition(
      tsxSource,
      tsxSource.indexOf("format(value: string): string"),
    );
    const implementation = await session.request("textDocument/implementation", {
      textDocument: { uri },
      position,
    });
    const location = firstLocation(implementation as never);
    assert.equal(location.uri, uri);
    assert.deepEqual(
      location.range.start,
      offsetToPosition(tsxSource, tsxSource.indexOf("format(value: string): string {")),
    );
    assert.ok(!location.uri.endsWith(".jsx.ts"), JSON.stringify(implementation));
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});

function writeVueShim(sourceDir: string): void {
  fs.writeFileSync(
    path.join(sourceDir, "vue-shim.d.ts"),
    `declare namespace JSX {
  interface IntrinsicElements {
    span: { children?: unknown }
  }
}

declare module "vue/jsx-runtime" {
  export const Fragment: unique symbol
  export function jsx(type: unknown, props: unknown): unknown
  export function jsxs(type: unknown, props: unknown): unknown
}
`,
    "utf8",
  );
}

type ImplementationLocation = { uri: string; range: LspRange };

function assertImplementationPacket(raw: unknown, expected: ImplementationLocation[]): void {
  if (expected.length === 0) {
    assert.equal(raw, null, "Vize returns None as null when no implementation remains");
    return;
  }
  const actual: unknown[] = Array.isArray(raw) ? raw : [raw];
  assert.equal(
    new Set(actual.map((entry) => JSON.stringify(entry))).size,
    actual.length,
    "implementation locations must not repeat",
  );
  const uri = (entry: unknown): string =>
    entry !== null && typeof entry === "object" && "uri" in entry
      ? String(entry.uri)
      : String(JSON.stringify(entry));
  const order = (left: unknown, right: unknown) => uri(left).localeCompare(uri(right));
  // Keep every returned field and both UTF-16 endpoints. Sorting only removes
  // the protocol's unspecified ordering; nothing is filtered or deduplicated.
  assert.deepEqual([...actual].sort(order), [...expected].sort(order));
}

function implementationLocation(file: string, text: string): ImplementationLocation {
  const start = text.indexOf("format(value: string): string {");
  assert.ok(start >= 0, "authored implementation marker is required");
  return {
    uri: pathToFileURL(file).href,
    range: {
      start: offsetToPosition(text, start),
      end: offsetToPosition(text, start + "format".length),
    },
  };
}

test("complete implementation packets reject missing, duplicate, stale and generated locations", () => {
  const text = "/* 雪😀 */ format(value: string): string { return value }\r\n";
  const expected = [
    implementationLocation("/workspace/Alpha.vue", text),
    implementationLocation("/workspace/Beta.tsx", text),
  ];
  assert.deepEqual(expected[0].range, {
    start: { line: 0, character: 10 },
    end: { line: 0, character: 16 },
  });
  assertImplementationPacket([...expected].reverse(), expected);
  assertImplementationPacket(expected[0], [expected[0]]);
  const stale = structuredClone(expected);
  stale[1].range.end.character++;
  const generated = structuredClone(expected);
  generated[0].uri += ".ts";
  const unexpected = structuredClone(expected);
  unexpected[0].uri = "file:///workspace/Other.vue";
  for (const raw of [
    null,
    [],
    [expected[0]],
    [expected[0], expected[0]],
    stale,
    generated,
    unexpected,
    [...expected, expected[0]],
  ])
    assert.throws(() => assertImplementationPacket(raw, expected));
  assertImplementationPacket(null, []);
  for (const raw of [undefined, [], expected[0], expected])
    assert.throws(() => assertImplementationPacket(raw, []));
});

function implementationText(
  kind: "vue" | "tsx",
  contract: string,
  phase: "initial" | "changed" | "removed",
): string {
  const name = kind === "vue" ? "Alpha" : "Beta";
  const script = [
    ...(phase === "changed" ? ["// unsaved 雪😀 revision", ""] : []),
    `import type { Formatter } from './${contract}'`,
    ...(phase === "removed"
      ? ["export const removed = true"]
      : [
          `export class ${name} implements Formatter {`,
          `  /* ${phase === "changed" ? "雪😀 changed" : "😀"} */ format(value: string): string { return value.toUpperCase() }`,
          "}",
        ]),
    kind === "vue" ? "export default {}" : "export const view = <span>implementation</span>",
  ];
  return (
    kind === "vue"
      ? ['<script lang="ts">', ...script, "</script>", "<template><span /></template>", ""]
      : [...script, ""]
  ).join("\r\n");
}

test("native implementation retains every SFC and TSX target through unsaved Unicode CRLF edits", async (t) => {
  const corsaPath = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "native implementation backend",
    "native implementation backend is required",
    true,
  )!;
  const require = createRequire(new URL("../package.json", import.meta.url));
  const vue = fs.realpathSync(
    process.env.VIZE_PROJECT_NAVIGATION_VUE_ROOT ??
      path.dirname(require.resolve("vue/package.json")),
  );
  fs.mkdirSync(testOutputRoot, { recursive: true });
  for (const queryKind of ["vue", "tsx"] as const) {
    const directory = fs.mkdtempSync(
      path.join(testOutputRoot, `whole-implementation-${queryKind}-`),
    );
    const sourceDir = path.join(directory, "src");
    fs.mkdirSync(sourceDir);
    fs.mkdirSync(path.join(directory, "node_modules"));
    fs.symlinkSync(vue, path.join(directory, "node_modules/vue"), "dir");
    fs.writeFileSync(
      path.join(directory, "vite.config.mjs"),
      `export default { vue: { typeChecker: ${JSON.stringify({ corsaPath, jsxTypecheck: true })} } }\n`,
    );
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "bundler",
          jsx: "preserve",
          jsxImportSource: "vue",
          allowImportingTsExtensions: true,
          noEmit: true,
          types: [],
        },
        include: ["src/**/*"],
      }),
    );
    const contract = `Contract.${queryKind}`;
    const contractScript = ["export interface Formatter {", "  format(value: string): string", "}"];
    const contractText = (
      queryKind === "vue"
        ? [
            '<script lang="ts">',
            ...contractScript,
            "export default {}",
            "</script>",
            "<template><span /></template>",
          ]
        : [...contractScript, "export const view = <span>contract</span>"]
    ).join("\r\n");
    const files = [
      { kind: "vue" as const, path: path.join(sourceDir, "Alpha.vue") },
      { kind: "tsx" as const, path: path.join(sourceDir, "Beta.tsx") },
    ].map((file) => ({
      ...file,
      initial: implementationText(file.kind, contract, "initial"),
      version: 1,
    }));
    const queryPath = path.join(sourceDir, contract);
    const queryUri = pathToFileURL(queryPath).href;
    const originals = new Map<string, string>([
      [queryPath, contractText],
      ...files.map((file) => [file.path, file.initial] as const),
    ]);
    for (const [file, text] of originals) fs.writeFileSync(file, text);
    const session = new LspSession();
    try {
      const init = (await session.initialize(directory, {
        editor: true,
        lint: false,
        typecheck: true,
      })) as { capabilities?: ImplementationCapabilities };
      assert.equal(init.capabilities?.implementationProvider, true);
      const barrier = async (uri: string, version: number) => {
        const packet = await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, uri) && params.version === version,
        );
        console.log(JSON.stringify({ oracle: "implementation-diagnostics", queryKind, packet }));
      };
      for (const [file, text] of originals) {
        const uri = pathToFileURL(file).href;
        session.notify("textDocument/didOpen", {
          textDocument: {
            uri,
            languageId: file.endsWith(".vue") ? "vue" : "typescriptreact",
            version: 1,
            text,
          },
        });
        await barrier(uri, 1);
      }
      const query = async (phase: string, expected: ImplementationLocation[]) => {
        const raw = await session.request("textDocument/implementation", {
          textDocument: { uri: queryUri },
          position: offsetToPosition(
            contractText,
            contractText.indexOf("format(value: string): string"),
          ),
        });
        console.log(
          JSON.stringify({ oracle: "whole-implementation", queryKind, phase, raw, expected }),
        );
        assertImplementationPacket(raw, expected);
        for (const [file, text] of originals)
          assert.equal(
            fs.readFileSync(file, "utf8"),
            text,
            "unsaved edits must leave original disk bytes intact",
          );
      };
      await query(
        "initial",
        files.map((file) => implementationLocation(file.path, file.initial)),
      );
      const change = async (file: (typeof files)[number], text: string) => {
        const uri = pathToFileURL(file.path).href;
        session.notify("textDocument/didChange", {
          textDocument: { uri, version: ++file.version },
          contentChanges: [{ text }],
        });
        await barrier(uri, file.version);
      };
      for (const changed of files) {
        for (const phase of ["changed", "removed", "initial"] as const) {
          const text =
            phase === "initial"
              ? changed.initial
              : implementationText(changed.kind, contract, phase);
          await change(changed, text);
          await query(
            `${changed.kind}-${phase === "initial" ? "restored" : phase}`,
            files.flatMap((file) =>
              file === changed
                ? phase === "removed"
                  ? []
                  : [implementationLocation(file.path, text)]
                : [implementationLocation(file.path, file.initial)],
            ),
          );
        }
      }
      for (const file of files)
        await change(file, implementationText(file.kind, contract, "removed"));
      await query("all-removed", []);
      const restored: ImplementationLocation[] = [];
      for (const file of files) {
        await change(file, file.initial);
        restored.push(implementationLocation(file.path, file.initial));
        await query(`${file.kind}-restored-after-all-removed`, restored);
      }
    } finally {
      try {
        await session.shutdown();
        await session.waitForClosed();
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  }
});
