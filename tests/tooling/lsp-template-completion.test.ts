import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { completionLabels, offsetToPosition } from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";

// Template completion contexts are parse-driven (typecheck:false, no corsa).
// These cases cover distinct cursor contexts that the smoke and roadmap suites
// do not exercise: opening-component-tag props, the built-in component list
// after a lone '<', the @vize: directive set inside an HTML comment, and CSS
// module class names after `$style.`. Scope-aware script/v-for completion
// already lives in lsp-roadmap-completion.test.ts and is not repeated here.
test("vize lsp surfaces template completion contexts", async (t) => {
  const testRootDir = path.join(testOutputRoot, "lsp-template-completion");
  fs.mkdirSync(testRootDir, { recursive: true });
  const workspaceDir = fs.mkdtempSync(path.join(testRootDir, "workspace-"));
  const session = new LspSession();

  try {
    const childSource = `<script setup lang="ts">
defineProps<{ diskOnly: string }>()
</script>
`;
    const openChildSource = `<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
</script>
<template><button /></template>
`;
    fs.writeFileSync(path.join(workspaceDir, "Child.vue"), childSource, "utf8");

    // Two spaces after <Child so the cursor sits inside the opening tag,
    // after the first space, before the self-closing slash.
    const parentSource = `<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child  />
</template>
`;
    const parentPath = path.join(workspaceDir, "Parent.vue");
    fs.writeFileSync(parentPath, parentSource, "utf8");

    const ltSource = `<template>
  <
</template>
`;
    const ltPath = path.join(workspaceDir, "Lt.vue");
    fs.writeFileSync(ltPath, ltSource, "utf8");

    const commentSource = `<template>
  <!-- @ -->
</template>
`;
    const commentPath = path.join(workspaceDir, "Comment.vue");
    fs.writeFileSync(commentPath, commentSource, "utf8");

    const styleSource = `<template><div :class="$style."></div></template>
<style module>
.box {}
.row {}
</style>
`;
    const stylePath = path.join(workspaceDir, "StyleMod.vue");
    fs.writeFileSync(stylePath, styleSource, "utf8");

    const authoringSource = `<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template>
  <button
    
  >
    {{ count }}
  </button>
</template>
`;
    const authoringPath = path.join(workspaceDir, "Authoring.vue");
    fs.writeFileSync(authoringPath, authoringSource, "utf8");

    const eventPrefixSource = `<template>
  <button @cli></button>
</template>
`;
    const eventPrefixPath = path.join(workspaceDir, "EventPrefix.vue");
    fs.writeFileSync(eventPrefixPath, eventPrefixSource, "utf8");

    const multilineEventSource = `<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template>
  <button
    @click="
      () => {
        co
      }
    "
  >
    {{ count }}
  </button>
</template>
`;
    const multilineEventPath = path.join(workspaceDir, "MultilineEvent.vue");
    fs.writeFileSync(multilineEventPath, multilineEventSource, "utf8");

    await session.initialize(workspaceDir, {
      editor: true,
      lint: true,
      typecheck: false,
    });

    const parentUri = pathToFileURL(parentPath).href;
    const childUri = pathToFileURL(path.join(workspaceDir, "Child.vue")).href;
    const ltUri = pathToFileURL(ltPath).href;
    const commentUri = pathToFileURL(commentPath).href;
    const styleUri = pathToFileURL(stylePath).href;
    const authoringUri = pathToFileURL(authoringPath).href;
    const eventPrefixUri = pathToFileURL(eventPrefixPath).href;
    const multilineEventUri = pathToFileURL(multilineEventPath).href;

    const openDocument = (uri: string, text: string): void => {
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri,
          languageId: "vue",
          version: 1,
          text,
        },
      });
    };

    openDocument(childUri, openChildSource);
    openDocument(parentUri, parentSource);
    openDocument(ltUri, ltSource);
    openDocument(commentUri, commentSource);
    openDocument(styleUri, styleSource);
    openDocument(authoringUri, authoringSource);
    openDocument(eventPrefixUri, eventPrefixSource);
    openDocument(multilineEventUri, multilineEventSource);

    await session.waitForNotification("textDocument/publishDiagnostics");

    await t.test(
      "inside an opening component tag surfaces the child's props plus directives",
      async () => {
        const offset = parentSource.indexOf("<Child ") + "<Child ".length;
        const response = await session.request("textDocument/completion", {
          textDocument: { uri: parentUri },
          position: offsetToPosition(parentSource, offset),
        });

        const labels = completionLabels(
          response as Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
        );
        assert.ok(labels.includes("label"), labels.join(", "));
        assert.ok(labels.includes("disabled"), labels.join(", "));
        assert.ok(!labels.includes("disk-only"), labels.join(", "));
        assert.ok(labels.includes("v-if"), labels.join(", "));
        assert.ok(labels.includes(":"), labels.join(", "));
        assert.ok(!labels.includes("Transition"), labels.join(", "));
      },
    );

    await t.test("after '<' surfaces built-in components and directive snippets", async () => {
      const offset = ltSource.indexOf("  <") + "  <".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: ltUri },
        position: offsetToPosition(ltSource, offset),
      });

      const labels = completionLabels(
        response as Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
      );
      for (const builtin of [
        "Transition",
        "Teleport",
        "Suspense",
        "KeepAlive",
        "component",
        "slot",
        "v-if",
      ]) {
        assert.ok(labels.includes(builtin), `missing ${builtin}; got ${labels.join(", ")}`);
      }
      // No <script>/imports here, so no user components should be offered.
      assert.ok(!labels.includes("Child"), labels.join(", "));
    });

    await t.test("inside an HTML comment returns only the @vize: directive set", async () => {
      const offset = commentSource.indexOf("@") + 1;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: commentUri },
        position: offsetToPosition(commentSource, offset),
      });

      const labels = completionLabels(
        response as Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
      );
      assert.deepEqual(labels, [
        "@vize:todo",
        "@vize:fixme",
        "@vize:expected",
        "@vize:docs",
        "@vize:ignore-start",
        "@vize:ignore-end",
        "@vize:level(warn)",
        "@vize:deprecated",
        "@vize:dev-only",
      ]);
      assert.ok(!labels.includes("v-if"), labels.join(", "));
    });

    await t.test("completion of $style. surfaces <style module> class names only", async () => {
      const offset = styleSource.indexOf("$style.") + "$style.".length;
      const response = (await session.request("textDocument/completion", {
        textDocument: { uri: styleUri },
        position: offsetToPosition(styleSource, offset),
      })) as
        | Array<{ label: string; kind?: number }>
        | { items?: Array<{ label: string; kind?: number }> }
        | null;

      const labels = completionLabels(response);
      assert.deepEqual(labels.slice().sort(), ["box", "row"]);

      const items = Array.isArray(response) ? response : (response?.items ?? []);
      assert.equal(items.length, 2, JSON.stringify(items));
      // CompletionItemKind.Field === 5 in the LSP spec.
      for (const item of items) {
        assert.equal(item.kind, 5, JSON.stringify(item));
      }
    });

    await t.test("inside a native opening tag surfaces HTML attrs and Vue events", async () => {
      const offset = authoringSource.indexOf("    \n  >") + "    ".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: authoringUri },
        position: offsetToPosition(authoringSource, offset),
      });

      const labels = completionLabels(response);
      for (const label of ["class", "type", "@click", "v-if"]) {
        assert.ok(labels.includes(label), `missing ${label}; got ${labels.join(", ")}`);
      }
      assert.ok(!labels.includes("count"), labels.join(", "));
      assert.ok(!labels.includes("Transition"), labels.join(", "));
    });

    await t.test("typing @cli in an opening tag offers @click", async () => {
      const offset = eventPrefixSource.indexOf("@cli") + "@cli".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: eventPrefixUri },
        position: offsetToPosition(eventPrefixSource, offset),
      });

      const labels = completionLabels(response);
      assert.ok(labels.includes("@click"), labels.join(", "));
      assert.ok(!labels.includes("v-if"), labels.join(", "));
      assert.ok(!labels.includes("class"), labels.join(", "));
    });

    await t.test("inside a multiline event handler surfaces script setup bindings", async () => {
      const offset = multilineEventSource.indexOf("        co") + "        co".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: multilineEventUri },
        position: offsetToPosition(multilineEventSource, offset),
      });

      const labels = completionLabels(response);
      assert.ok(labels.includes("count"), labels.join(", "));
      assert.ok(!labels.includes("v-if"), labels.join(", "));
      assert.ok(!labels.includes("Transition"), labels.join(", "));
    });
  } finally {
    await session.shutdown();
    fs.rmSync(workspaceDir, { recursive: true, force: true });
    fs.rmSync(testRootDir, { recursive: true, force: true });
  }
});
