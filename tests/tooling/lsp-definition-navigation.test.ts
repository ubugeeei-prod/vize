import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { firstLocation, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// Navigation suite for the non-corsa (parse-driven) definition and references
// paths: plain-const template definition, cross-file component-prop definition,
// and in-file find-references with the includeDeclaration toggle. These live
// apart from lsp-smoke.test.ts, which already covers component-tag definition
// and the ref()/Corsa-backed coordinate paths. typecheck is disabled so no
// type checker is involved and the spans stay deterministic.

type Location = {
  uri: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
};

type DefinitionResponse = Location | Location[] | null;

test("vize lsp navigates definitions and references without the type checker", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-definition-navigation");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: false,
    });

    await t.test(
      "template interpolation of a plain const resolves to its script declaration",
      async () => {
        const source = `<script setup lang="ts">
const userName = 'ada'
</script>

<template>
  <p>{{ userName }}</p>
</template>
`;
        const filePath = path.join(workspaceDir, "PlainConst.vue");
        const uri = pathToFileURL(filePath).href;
        fs.writeFileSync(filePath, source, "utf8");

        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, uri),
        );

        const usageOffset = source.lastIndexOf("userName") + "userName".length;
        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri },
          position: offsetToPosition(source, usageOffset),
        })) as DefinitionResponse;

        assert.ok(definition, "definition should resolve a plain-const binding");
        const location = firstLocation(definition);
        assert.equal(location.uri, uri);

        const declStart = offsetToPosition(source, source.indexOf("userName"));
        assert.deepEqual(location.range.start, declStart);
        assert.deepEqual(location.range.start, { line: 1, character: 6 });
        assert.equal(
          location.range.end.character,
          location.range.start.character + "userName".length,
        );
      },
    );

    await t.test(
      "component prop attribute resolves cross-file into the child's defineProps",
      async () => {
        const childSource = `<script setup lang="ts">
defineProps<{
  label: string;
  disabled?: boolean;
}>()
</script>

<template>
  <button>{{ label }}</button>
</template>
`;
        const childPath = path.join(workspaceDir, "PropChild.vue");
        const childUri = pathToFileURL(childPath).href;
        fs.writeFileSync(childPath, childSource, "utf8");

        const parentSource = `<script setup lang="ts">
import PropChild from './PropChild.vue'
const msg = 'hi'
</script>

<template>
  <PropChild :label="msg" />
</template>
`;
        const parentPath = path.join(workspaceDir, "PropParent.vue");
        const parentUri = pathToFileURL(parentPath).href;
        fs.writeFileSync(parentPath, parentSource, "utf8");

        session.notify("textDocument/didOpen", {
          textDocument: { uri: childUri, languageId: "vue", version: 1, text: childSource },
        });
        session.notify("textDocument/didOpen", {
          textDocument: { uri: parentUri, languageId: "vue", version: 1, text: parentSource },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, parentUri),
        );

        // Anchor inside the `label` of `:label="msg"` in the parent template.
        const labelOffset = parentSource.indexOf(":label") + ":label".length - 1;
        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri: parentUri },
          position: offsetToPosition(parentSource, labelOffset),
        })) as DefinitionResponse;

        assert.ok(definition, "prop definition should resolve cross-file");
        const location = firstLocation(definition);
        assert.equal(location.uri, childUri);

        const declStart = offsetToPosition(childSource, childSource.indexOf("label: string"));
        assert.deepEqual(location.range.start, declStart);
        assert.deepEqual(location.range.start, { line: 2, character: 2 });
        assert.equal(location.range.end.character, location.range.start.character + "label".length);
      },
    );

    await t.test(
      "find-references covers all in-file uses and includeDeclaration toggles the count",
      async () => {
        const source = `<script setup lang="ts">
const price = 10
const doubled = price * 2
</script>

<template>
  {{ price }}
</template>
`;
        const filePath = path.join(workspaceDir, "References.vue");
        const uri = pathToFileURL(filePath).href;
        fs.writeFileSync(filePath, source, "utf8");

        session.notify("textDocument/didOpen", {
          textDocument: { uri, languageId: "vue", version: 1, text: source },
        });
        await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
          isDiagnosticsForUri(params, uri),
        );

        // Anchor on the script-side use inside `const doubled = price * 2`.
        const usageOffset = source.indexOf("= price") + "= ".length + 1;
        const declStart = offsetToPosition(source, source.indexOf("const price") + "const ".length);
        assert.deepEqual(declStart, { line: 1, character: 6 });

        const withDeclaration = (await session.request("textDocument/references", {
          textDocument: { uri },
          position: offsetToPosition(source, usageOffset),
          context: { includeDeclaration: true },
        })) as Location[];

        assert.ok(Array.isArray(withDeclaration), JSON.stringify(withDeclaration));
        assert.equal(withDeclaration.length, 4);
        assert.ok(
          withDeclaration.every((reference) => reference.uri === uri),
          JSON.stringify(withDeclaration),
        );
        const hasDeclaration = (locations: Location[]): boolean =>
          locations.some(
            (location) =>
              location.range.start.line === declStart.line &&
              location.range.start.character === declStart.character,
          );
        assert.ok(hasDeclaration(withDeclaration), JSON.stringify(withDeclaration));

        const withoutDeclaration = (await session.request("textDocument/references", {
          textDocument: { uri },
          position: offsetToPosition(source, usageOffset),
          context: { includeDeclaration: false },
        })) as Location[];

        assert.ok(Array.isArray(withoutDeclaration), JSON.stringify(withoutDeclaration));
        assert.equal(withoutDeclaration.length, 3);
        assert.ok(
          withoutDeclaration.every((reference) => reference.uri === uri),
          JSON.stringify(withoutDeclaration),
        );
        assert.equal(hasDeclaration(withoutDeclaration), false, JSON.stringify(withoutDeclaration));

        // includeDeclaration:false is exactly the includeDeclaration:true set with
        // the declaration occurrence removed.
        const serialize = (location: Location): string =>
          `${location.range.start.line}:${location.range.start.character}`;
        const expectedWithout = withDeclaration
          .filter(
            (location) =>
              !(
                location.range.start.line === declStart.line &&
                location.range.start.character === declStart.character
              ),
          )
          .map(serialize)
          .sort();
        assert.deepEqual(withoutDeclaration.map(serialize).sort(), expectedWithout);
      },
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
