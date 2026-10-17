import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  completionLabels,
  isDiagnosticsForUri,
  offsetToPosition,
} from "./support/lsp/assertions.ts";
import { testOutputRoot } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
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
    // Both props are read in the template: `vue/no-unused-properties` is
    // enabled here, and the assertions below expect a clean child document.
    const openChildSource = `<script setup lang="ts">
defineProps<{ label: string; disabled?: boolean }>()
</script>
<template><button>{{ label }}{{ disabled }}</button></template>
`;
    const changedChildSource = openChildSource
      .replaceAll("disabled", "loading")
      .replace("loading?: boolean", "loading?: number");
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

    const styleSource = `<template><div data-note="🌱" :class="$style."></div></template>
<style module>
.box {}
.row {}
</style>
`;
    const stylePath = path.join(workspaceDir, "Style 模块.vue");
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

    await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => !isDiagnosticsForUri(params, styleUri),
    );

    await t.test("component prop completion tracks unsaved child edits and repair", async () => {
      const offset = parentSource.indexOf("<Child ") + "<Child ".length;
      const requestLabels = async (): Promise<string[]> => {
        const response = await session.request("textDocument/completion", {
          textDocument: { uri: parentUri },
          position: offsetToPosition(parentSource, offset),
        });
        return completionLabels(
          response as Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
        );
      };

      const labels = await requestLabels();
      assert.ok(labels.includes("label"), labels.join(", "));
      assert.ok(labels.includes("disabled"), labels.join(", "));
      assert.ok(!labels.includes("disk-only"), labels.join(", "));
      assert.ok(labels.includes("v-if"), labels.join(", "));
      assert.ok(labels.includes(":"), labels.join(", "));
      assert.ok(!labels.includes("Transition"), labels.join(", "));

      session.notify("textDocument/didChange", {
        textDocument: { uri: childUri, version: 2 },
        contentChanges: [{ text: changedChildSource }],
      });
      const changedPublish = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, childUri) && params.version === 2,
      )) as PublishDiagnosticsParams;
      assert.deepEqual(changedPublish.diagnostics, []);

      const changedLabels = await requestLabels();
      assert.ok(changedLabels.includes("label"), changedLabels.join(", "));
      assert.ok(changedLabels.includes("loading"), changedLabels.join(", "));
      assert.ok(!changedLabels.includes("disabled"), changedLabels.join(", "));
      assert.ok(!changedLabels.includes("disk-only"), changedLabels.join(", "));

      session.notify("textDocument/didChange", {
        textDocument: { uri: childUri, version: 3 },
        contentChanges: [{ text: openChildSource }],
      });
      const repairedPublish = (await session.waitForNotification(
        "textDocument/publishDiagnostics",
        (params) => isDiagnosticsForUri(params, childUri) && params.version === 3,
      )) as PublishDiagnosticsParams;
      assert.deepEqual(repairedPublish.diagnostics, []);

      const repairedLabels = await requestLabels();
      assert.ok(repairedLabels.includes("label"), repairedLabels.join(", "));
      assert.ok(repairedLabels.includes("disabled"), repairedLabels.join(", "));
      assert.ok(!repairedLabels.includes("loading"), repairedLabels.join(", "));
      assert.ok(!repairedLabels.includes("disk-only"), repairedLabels.join(", "));
    });

    await t.test("after '<' surfaces built-in components and native tag names", async () => {
      const offset = ltSource.indexOf("  <") + "  <".length;
      const response = await session.request("textDocument/completion", {
        textDocument: { uri: ltUri },
        position: offsetToPosition(ltSource, offset),
      });

      const labels = completionLabels(
        response as Array<{ label: string }> | { items?: Array<{ label: string }> } | null,
      );
      const expected = [
        "KeepAlive",
        "Suspense",
        "Teleport",
        "Transition",
        "TransitionGroup",
        "component",
        "div",
        "slot",
        "span",
        "svg",
      ];
      const required = new Set(expected);
      assert.deepEqual(labels.filter((label) => required.has(label)).sort(), expected);
      // Tag-name positions accept names, while directives belong to attributes.
      const excluded = new Set(["Child", "v-if", "v-for"]);
      assert.deepEqual(
        labels.filter((label) => excluded.has(label)),
        [],
      );
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
      const requestItems = async (text: string, names: string[]) => {
        const offset = text.indexOf("$style.") + "$style.".length;
        const position = offsetToPosition(text, offset);
        assert.deepEqual(position, { line: 0, character: 45 });
        const response = await session.request("textDocument/completion", {
          textDocument: { uri: styleUri },
          position,
        });
        // The whole authored items have no textEdit or generated-source range.
        assert.deepEqual(
          response,
          names.length
            ? names.map((name) => ({
                label: name,
                kind: 5,
                detail: "CSS module class",
                documentation: {
                  kind: "markdown",
                  value: `\`${name}\` from \`<style module>\`.`,
                },
                sortText: `0${name}`,
              }))
            : null,
        );
        assert.equal(fs.readFileSync(stylePath, "utf8"), styleSource);
      };
      const waitForStyle = (version: number) =>
        session.waitForNotification(
          "textDocument/publishDiagnostics",
          (params) => isDiagnosticsForUri(params, styleUri) && params.version === version,
        );
      const changeStyle = async (text: string, version: number) => {
        session.notify("textDocument/didChange", {
          textDocument: { uri: styleUri, version },
          contentChanges: [{ text }],
        });
        await waitForStyle(version);
      };
      await waitForStyle(1);
      await requestItems(styleSource, ["box", "row"]);

      const template = styleSource.slice(0, styleSource.indexOf("\n"));
      const changed = `${template}
<style module>
/* .commentOnly {} */
.active:global(.loaded) { content: ".valueOnly"; background: url("./asset.png"); }
:global(.external) .root {}
[data-kind=".attributeOnly"] .updated {}
:global .globalOnly, :local(.after) {}
._local {} .root {}
</style>
<style>.plainOnly {}</style>
<style module="theme">.themeOnly {}</style>
`;
      await changeStyle(changed, 2);
      await requestItems(changed, ["_local", "active", "after", "root", "updated"]);
      await changeStyle(styleSource, 3);
      await requestItems(styleSource, ["box", "row"]);

      fs.writeFileSync(path.join(workspaceDir, "foreign.css"), ".externalOnly {}\n");
      const unsupported = [
        "<style module>:global(.globalOnly) {}</style>",
        "<style module>.root) {}</style>",
        "<style>.plainOnly {}</style>",
        '<style module="theme">.themeOnly {}</style>',
        '<style module src="./foreign.css"></style>',
        '<style module lang="scss">.scssOnly {}</style>',
      ];
      for (const [index, style] of unsupported.entries()) {
        const text = `${template}\n${style}\n`;
        await changeStyle(text, index + 4);
        await requestItems(text, []);
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
