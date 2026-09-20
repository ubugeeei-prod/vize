import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
  workspace,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

function createProject(options: Record<string, unknown> = {}): string {
  const directory = workspace("jsx-module-contract-");
  fs.writeFileSync(
    path.join(directory, "vize.config.json"),
    JSON.stringify({ typeChecker: { jsxTypecheck: true } }),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        skipLibCheck: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        jsx: "preserve",
        jsxImportSource: "vue",
        allowImportingTsExtensions: true,
        noEmit: true,
        paths: { "@/*": ["./*"] },
        ...options,
      },
      include: ["**/*.vue", "**/*.ts", "**/*.tsx"],
    }),
  );
  fs.writeFileSync(
    path.join(directory, "Child.tsx"),
    `export default (props: { count: number }) => <div>{props.count}</div>;\n`,
  );
  fs.writeFileSync(
    path.join(directory, "entry.tsx"),
    `import Child from './Child.js';
export const marker = 123;
export default (props: { count: number }) => <Child count={props.count} />;
`,
  );
  fs.writeFileSync(
    path.join(directory, "barrel.ts"),
    "export { default, marker } from './entry.js';\n",
  );
  fs.mkdirSync(path.join(directory, "nested"));
  fs.writeFileSync(
    path.join(directory, "nested/index.tsx"),
    "export { default, marker } from '../entry';\n",
  );
  return directory;
}

async function compare(directory: string, count: number): Promise<void> {
  const expected = vueTscDiagnostics(directory).sort(compareIdentity);
  assert.equal(expected.length, count, JSON.stringify(expected));
  assert.deepEqual(
    (await check(directory)).map(diagnosticIdentity).sort(compareIdentity),
    expected,
  );
}

test("lowered TSX modules preserve relative, alias, barrel and directory import contracts", async () => {
  const directory = createProject();
  try {
    for (const specifier of [
      "./entry",
      "./entry.tsx",
      "./entry.js",
      "@/entry",
      "./barrel",
      "./nested",
      path.join(directory, "entry.tsx").replaceAll("\\", "/"),
    ]) {
      const source = `<script setup lang="ts">
import Entry, { marker } from '${specifier}';
marker.toFixed();
</script><template><Entry :count="1" /></template>`;
      fs.writeFileSync(path.join(directory, "App.vue"), source);
      fs.writeFileSync(
        path.join(directory, "consumer.ts"),
        `import Entry from '${specifier}';
const props: Parameters<typeof Entry>[0] = { count: 1 };
void props;
const lazy = import('${specifier}');
void lazy;
`,
      );
      await compare(directory, 0);
      fs.writeFileSync(
        path.join(directory, "App.vue"),
        source
          .replace("marker.toFixed()", "marker.toUpperCase()")
          .replace(':count="1"', 'count="bad"'),
      );
      await compare(directory, 2);
    }
    fs.rmSync(path.join(directory, "App.vue"));
    fs.writeFileSync(path.join(directory, "consumer.ts"), "export { default } from './entry';\n");
    fs.writeFileSync(
      path.join(directory, "entry.tsx"),
      "import Child from './Child.js';\nexport const marker = 123;\nexport default () => <Child count=\"bad\" />;\n",
    );
    await compare(directory, 1);
    fs.writeFileSync(
      path.join(directory, "entry.tsx"),
      "import Child from './Child.js';\nexport const marker = 123;\nexport default () => <Child count={1} />;\n",
    );
    await compare(directory, 0);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("NodeNext keeps runtime extension imports and rejects invalid extensionless ESM", async () => {
  const directory = createProject({ module: "NodeNext", moduleResolution: "NodeNext" });
  try {
    fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}');
    fs.rmSync(path.join(directory, "nested"), { recursive: true });
    const file = path.join(directory, "consumer.ts");
    fs.writeFileSync(
      file,
      "import Entry from './entry.js';\nexport const props: Parameters<typeof Entry>[0] = { count: 1 };\n",
    );
    await compare(directory, 0);
    fs.writeFileSync(file, "import Entry from './entry';\nvoid Entry;\n");
    const expected = vueTscDiagnostics(directory);
    assert.equal(expected.length, 1, JSON.stringify(expected));
    const actual = await check(directory);
    assert.equal(actual.length, 1, JSON.stringify(actual));
    assert.deepEqual(expected, [{ file: "consumer.ts", line: 1, column: 19, code: 2835 }]);
    // The private mirror cannot suggest the absent authored .tsx filename.
    // Track that suggestion difference explicitly; never accept a clean import.
    assert.deepEqual(actual.map(diagnosticIdentity), [
      { file: "consumer.ts", line: 1, column: 19, code: 2834 },
    ]);
    fs.writeFileSync(file, "import Entry from './entry.js';\nvoid Entry;\n");
    await compare(directory, 0);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("Vue importers retain exact TSX contracts through unsaved edits and close", async () => {
  const directory = createProject();
  const session = new LspSession();
  const app = path.join(directory, "App.vue");
  const entry = path.join(directory, "entry.tsx");
  const appUri = pathToFileURL(app).href;
  const entryUri = pathToFileURL(entry).href;
  const source = `<script setup lang="ts">
import Entry from './barrel';
</script><template><Entry :count="1" /></template>`;
  const component = `export const marker = 123;
export default (props: { count: number }) => <div>{props.count}</div>;`;
  const wait = async (uri: string, version: number) =>
    (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => isDiagnosticsForUri(params, uri) && params.version === version,
    )) as PublishDiagnosticsParams;
  try {
    fs.writeFileSync(app, source);
    fs.writeFileSync(entry, component);
    await compare(directory, 0);
    await session.initialize(directory, { editor: true, lint: false, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri: appUri, languageId: "vue", version: 1, text: source },
    });
    assert.deepEqual((await wait(appUri, 1)).diagnostics, []);
    session.notify("textDocument/didOpen", {
      textDocument: {
        uri: entryUri,
        languageId: "typescriptreact",
        version: 1,
        text: component.replace("count: number", "count: string"),
      },
    });
    assert.deepEqual((await wait(entryUri, 1)).diagnostics, []);
    const broken = await wait(appUri, 1);
    const start = source.indexOf(":count") + 1;
    assert.deepEqual(
      broken.diagnostics.map(({ code, range }) => ({ code: Number(code), range })),
      [
        {
          code: 2322,
          range: {
            start: offsetToPosition(source, start),
            end: offsetToPosition(source, start + "count".length),
          },
        },
      ],
    );
    session.notify("textDocument/didChange", {
      textDocument: { uri: entryUri, version: 2 },
      contentChanges: [{ text: component }],
    });
    assert.deepEqual((await wait(entryUri, 2)).diagnostics, []);
    assert.deepEqual((await wait(appUri, 1)).diagnostics, []);
    session.notify("textDocument/didChange", {
      textDocument: { uri: entryUri, version: 3 },
      contentChanges: [{ text: component.replace("count: number", "count: string") }],
    });
    assert.deepEqual((await wait(entryUri, 3)).diagnostics, []);
    assert.deepEqual((await wait(appUri, 1)).diagnostics, broken.diagnostics);
    session.notify("textDocument/didClose", { textDocument: { uri: entryUri } });
    assert.deepEqual((await wait(appUri, 1)).diagnostics, []);
    await compare(directory, 0);
  } finally {
    await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
