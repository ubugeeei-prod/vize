import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  completionLabels,
  firstLocation,
  hoverToText,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

type Location = {
  uri: string;
  range: {
    start: { line: number; character: number };
    end: { line: number; character: number };
  };
};

type DefinitionResponse = Location | Location[] | null;

test("vize lsp gates production Vue authoring boundaries", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-editor-production-gates");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: false,
    });

    const source = `<script setup lang="ts">
import { ref } from 'vue'

const count = ref(0)
const isReady = ref(false)
</script>

<template>
  <button
    
    class="co"
    @click="
      () => {
        count
      }
    "
    :disabled="isReady"
  >
    {{ count }}
  </button>
</template>
`;
    const filePath = path.join(workspaceDir, "ProductionGates.vue");
    const uri = pathToFileURL(filePath).href;
    fs.writeFileSync(filePath, source, "utf8");

    session.notify("textDocument/didOpen", {
      textDocument: {
        uri,
        languageId: "vue",
        version: 1,
        text: source,
      },
    });

    await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
      isDiagnosticsForUri(params, uri),
    );

    await t.test("opening tag completion offers attributes and directives only", async () => {
      const offset = source.indexOf("    \n    class") + "    ".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri },
        position: offsetToPosition(source, offset),
      });

      const labels = completionLabels(response);
      for (const label of ["class", "type", "@click", "v-if"]) {
        assert.ok(labels.includes(label), `missing ${label}; got ${labels.join(", ")}`);
      }
      assert.ok(!labels.includes("count"), labels.join(", "));
      assert.ok(!labels.includes("isReady"), labels.join(", "));
      assert.ok(!labels.includes("Transition"), labels.join(", "));
    });

    await t.test(
      "static attribute values do not leak script or directive completions",
      async () => {
        const offset = source.indexOf('class="co') + 'class="co'.length;
        const response = await session.request("textDocument/completion", {
          textDocument: { uri },
          position: offsetToPosition(source, offset),
        });

        assert.deepEqual(completionLabels(response), []);
      },
    );

    await t.test(
      "event handler expressions offer bindings but not authoring directives",
      async () => {
        const offset = source.indexOf("        count") + "        cou".length;
        const response = await session.request("textDocument/completion", {
          textDocument: { uri },
          position: offsetToPosition(source, offset),
        });

        const labels = completionLabels(response);
        assert.ok(labels.includes("count"), labels.join(", "));
        assert.ok(labels.includes("isReady"), labels.join(", "));
        assert.ok(!labels.includes("v-if"), labels.join(", "));
        assert.ok(!labels.includes("@click"), labels.join(", "));
        assert.ok(!labels.includes("class"), labels.join(", "));
      },
    );

    await t.test("directive prefix completion stays on the event/directive surface", async () => {
      const prefixSource = `<template><button @></button></template>`;
      const prefixPath = path.join(workspaceDir, "DirectivePrefix.vue");
      const prefixUri = pathToFileURL(prefixPath).href;
      fs.writeFileSync(prefixPath, prefixSource, "utf8");
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri: prefixUri,
          languageId: "vue",
          version: 1,
          text: prefixSource,
        },
      });
      await session.waitForNotification("textDocument/publishDiagnostics", (params) =>
        isDiagnosticsForUri(params, prefixUri),
      );

      const offset = prefixSource.indexOf("@") + 1;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: prefixUri },
        position: offsetToPosition(prefixSource, offset),
      });

      const labels = completionLabels(response);
      assert.ok(labels.includes("@click"), labels.join(", "));
      assert.ok(!labels.includes("class"), labels.join(", "));
      assert.ok(!labels.includes("v-if"), labels.join(", "));
      assert.ok(!labels.includes("Transition"), labels.join(", "));
    });

    await t.test("rich hover covers native attrs, events, and template bindings", async () => {
      const disabledAttrOffset = source.indexOf(":disabled") + ":disabled".length;
      const disabledHover = (await session.request("textDocument/hover", {
        textDocument: { uri },
        position: offsetToPosition(source, disabledAttrOffset),
      })) as { contents?: unknown } | null;
      const disabledText = hoverToText(disabledHover);
      assert.match(disabledText, /disabled on <button>/);
      assert.match(disabledText, /HTML attribute/);
      assert.match(disabledText, /MDN reference/);

      const clickOffset = source.indexOf("@click") + "@click".length;
      const clickHover = (await session.request("textDocument/hover", {
        textDocument: { uri },
        position: offsetToPosition(source, clickOffset),
      })) as { contents?: unknown } | null;
      const clickText = hoverToText(clickHover);
      assert.match(clickText, /Vue event listener/);
      assert.match(clickText, /\$event/);

      const interpolationOffset = source.lastIndexOf("count }}") + "count".length;
      const bindingHover = (await session.request("textDocument/hover", {
        textDocument: { uri },
        position: offsetToPosition(source, interpolationOffset),
      })) as { contents?: unknown } | null;
      const bindingText = hoverToText(bindingHover);
      assert.match(bindingText, /count/);
      assert.match(bindingText, /Template binding from script/);
      assert.match(bindingText, /automatically unwrapped|template scope/i);
    });

    await t.test(
      "definition and references include event handlers and interpolations",
      async () => {
        const eventUseOffset = source.indexOf("        count") + "        count".length;
        const eventUsePosition = offsetToPosition(source, eventUseOffset);
        const declarationOffset = source.indexOf("count = ref");
        const declarationPosition = offsetToPosition(source, declarationOffset);

        const definition = (await session.request("textDocument/definition", {
          textDocument: { uri },
          position: eventUsePosition,
        })) as DefinitionResponse;

        assert.ok(definition, "definition should resolve from an inline event handler");
        const location = firstLocation(definition);
        assert.equal(location.uri, uri);
        assert.deepEqual(location.range.start, declarationPosition);

        const references = (await session.request("textDocument/references", {
          textDocument: { uri },
          position: eventUsePosition,
          context: { includeDeclaration: true },
        })) as Location[];

        const starts = references.map((reference) => reference.range.start);
        assert.ok(
          starts.some(
            (start) =>
              start.line === declarationPosition.line &&
              start.character === declarationPosition.character,
          ),
          JSON.stringify(references),
        );
        assert.ok(
          starts.some(
            (start) =>
              start.line === eventUsePosition.line &&
              start.character === eventUsePosition.character - "count".length,
          ),
          JSON.stringify(references),
        );

        const interpolationStart = offsetToPosition(source, source.lastIndexOf("{{ count }}") + 3);
        assert.ok(
          starts.some(
            (start) =>
              start.line === interpolationStart.line &&
              start.character === interpolationStart.character,
          ),
          JSON.stringify(references),
        );
      },
    );
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
