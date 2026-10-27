import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import ts from "typescript";
import { createVizeLanguagePlugin } from "../src/index.js";
import { publicInstanceReference } from "./public-instance-reference.js";
import {
  directory,
  workspace,
  evidence,
  packets,
  inputs,
  timings,
  hash,
  normalize,
  query,
  fixture,
} from "./service-harness.js";

const basic = `<script setup lang="ts">\nimport { suffix } from './lib';\nconst title: string = 'hi';\nconst count: number = 'bad';\nconst record = { alpha: 1, beta: 'two' };\nrecord.alpha;\nfunction label(value: string) { return value + suffix; }\n</script>\n<template><p>{{ title.toUpperCase() }} {{ label(title) }} {{ absent }}</p></template>\n`;

void test("actual native projection through Volar and TypeScript: complete packets and lifecycle", () => {
  assert.equal(ts.version, "6.0.3");
  const f = fixture("basic", { "App.vue": basic, "lib.ts": "export const suffix = '!';\n" });
  const file = f.path("App.vue");
  const ls = f.languageService;
  const at = (needle, delta = 0) => f.text("App.vue").indexOf(needle) + delta;
  try {
    const diagnostics = query("basic-diagnostics", { file }, () => [
      ...ls.getSyntacticDiagnostics(file),
      ...ls.getSemanticDiagnostics(file),
    ]);
    assert(
      diagnostics.some(
        (d) => d.code === 2322 && basic.slice(d.start, d.start + d.length) === "count",
      ),
    );
    assert(diagnostics.some((d) => d.code === 2339 || d.code === 2304));
    for (const [name, position] of [
      ["script-hover", at("title: string")],
      ["template-hover", at("title.toUpperCase")],
    ]) {
      const hover = query(name, { file, position }, () =>
        ls.getQuickInfoAtPosition(file, position),
      );
      assert(hover && ts.displayPartsToString(hover.displayParts).includes("string"));
    }
    const position = at("record.alpha;", "record.".length);
    const completion = query("property-completion", { file, position }, () =>
      ls.getCompletionsAtPosition(file, position, {}),
    );
    assert.deepEqual(completion.entries.map((entry) => entry.name).sort(), ["alpha", "beta"]);
    const definitionPosition = at("value + suffix", "value + ".length);
    const definition = query("import-definition", { file, position: definitionPosition }, () =>
      ls.getDefinitionAndBoundSpan(file, definitionPosition),
    );
    assert(definition.definitions.some((d) => d.fileName === f.path("lib.ts")));
    const title = at("title: string");
    const references = query("title-references", { file, position: title }, () =>
      ls.findReferences(file, title),
    );
    assert(
      references
        .flatMap((r) => r.references)
        .some((r) => r.textSpan.start === at("title.toUpperCase")),
    );
    query("title-rename-info", { file, position: title }, () => ls.getRenameInfo(file, title));
    const rename = query("title-rename", { file, position: title }, () =>
      ls.findRenameLocations(file, title, false, false, true),
    );
    assert(rename.some((r) => r.textSpan.start === at("title.toUpperCase")));
    const root = f.language.scripts.get(file).generated.root;
    assert.equal(root.source, basic);
    assert(
      root.projection.mappings.every(
        (row, index) =>
          row[5] !== 0 || root.embeddedCodes[0].mappings[index].data.navigation === false,
      ),
    );
    inputs.push({ name: "basic-native-projection", projection: root.projection });
    f.capture("initial");
    const oldHover = ls.getQuickInfoAtPosition(file, title);
    assert.deepEqual(
      query("unchanged-repeat-hover", { file, position: title }, () =>
        ls.getQuickInfoAtPosition(file, title),
      ),
      oldHover,
      "unchanged snapshot is stable",
    );
    f.update("App.vue", basic.replace("const title: string = 'hi'", "const title: boolean = true"));
    const updated = query("unsaved-update-hover", { file, position: title }, () =>
      ls.getQuickInfoAtPosition(file, title),
    );
    assert(ts.displayPartsToString(updated.displayParts).includes("boolean"));
    query("unsaved-update-diagnostics", { file }, () => ls.getSemanticDiagnostics(file));
    f.capture("unsaved-update");
    f.delete("lib.ts");
    const missing = query("delete-import-diagnostics", { file }, () =>
      ls.getSemanticDiagnostics(file),
    );
    assert(missing.some((d) => d.code === 2307));
    f.update("lib.ts", "export const suffix = 'restored';\n");
    const restored = query("recreate-import-diagnostics", { file }, () =>
      ls.getSemanticDiagnostics(file),
    );
    assert(!restored.some((d) => d.code === 2307));
    f.capture("restored-import");
    f.delete("App.vue");
    assert.equal(f.language.scripts.get(file), undefined);
  } finally {
    ls.dispose();
  }
});

void test("Unicode, CRLF, directives, invalid syntax, generic slots and resolver options", () => {
  const unicode =
    '<script setup lang="ts">\r\nconst 日本 = "😀";\r\nconst 数: number = "bad";\r\n</script>\r\n<template>{{ 日本 }}</template>\r\n';
  const f = fixture("unicode", { "Unicode.vue": unicode });
  const file = f.path("Unicode.vue");
  try {
    const diagnostics = query("unicode-diagnostics", { file }, () =>
      f.languageService.getSemanticDiagnostics(file),
    );
    assert(
      diagnostics.some(
        (d) => d.code === 2322 && unicode.slice(d.start, d.start + d.length) === "数",
      ),
    );
    const position = unicode.lastIndexOf("日本");
    const hover = query("unicode-template-hover", { file, position }, () =>
      f.languageService.getQuickInfoAtPosition(file, position),
    );
    assert(hover && hover.textSpan.start === position && hover.textSpan.length === 2);
    const numberPosition = unicode.indexOf("数: number");
    assert.deepEqual(
      query("unicode-authored-line-column", { file, position: numberPosition }, () =>
        f.languageService.toLineColumnOffset(file, numberPosition),
      ),
      { line: 2, character: 6 },
    );
    f.capture("unicode-crlf");
  } finally {
    f.languageService.dispose();
  }

  // Directives own their target node, not every descendant of an element.
  const directives = `<script setup lang="ts">const valid = 'ok';</script>\n<template>\n<!-- @vue-ignore -->{{ ignoredMissing }}\n<!-- @vue-expect-error -->{{ expectedMissing }}\n<!-- @vue-expect-error -->{{ valid }}\n<p>{{ realMissing }}</p>\n<!-- @vue-ignore --><p :id="valid">{{ childMissing }}</p>\n</template>`;
  const d = fixture("directives", { "Directives.vue": directives });
  try {
    const result = query("directive-diagnostics", {}, () =>
      d.languageService.getSemanticDiagnostics(d.path("Directives.vue")),
    );
    assert.equal(result.filter((diagnostic) => diagnostic.code === 4).length, 1);
    assert(
      !result.some((diagnostic) =>
        /ignoredMissing|expectedMissing/.test(
          ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
        ),
      ),
    );
    d.capture("directives");
    assert(
      result.some((diagnostic) =>
        ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n").includes("realMissing"),
      ),
    );
    assert(
      result.some((diagnostic) =>
        ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n").includes("childMissing"),
      ),
    );
    d.update(
      "Directives.vue",
      '<script setup lang="ts">const x = ;</script><template>{{ x }}</template>',
    );
    const invalid = query("invalid-code-diagnostics", {}, () => [
      ...d.languageService.getSyntacticDiagnostics(d.path("Directives.vue")),
      ...d.languageService.getSemanticDiagnostics(d.path("Directives.vue")),
    ]);
    assert(invalid.length > 0);
    d.capture("invalid");
    d.update("Directives.vue", '<script setup lang="ts">const value = 1;');
    const malformed = query("malformed-sfc-diagnostics", {}, () =>
      d.languageService.getSemanticDiagnostics(d.path("Directives.vue")),
    );
    assert(malformed.some((diagnostic) => diagnostic.code === 100001));
    d.capture("malformed-sfc");
  } finally {
    d.languageService.dispose();
  }

  const child =
    '<script setup lang="ts" generic="T extends { id: number }">\ndefineProps<{ item: T }>();\ndefineSlots<{ default(props: { item: T }): unknown }>();\n</script>\n<template><slot :item="item" /></template>';
  const parent =
    '<script setup lang="ts">import Child from "./Child.vue";</script>\n<template><Child :item="{ id: 1 }" v-slot="{ item }">{{ item.id }}</Child></template>';
  const g = fixture("generic-slots", { "Child.vue": child, "Parent.vue": parent });
  try {
    const diagnostics = query("generic-slot-diagnostics", {}, () =>
      g.languageService.getSemanticDiagnostics(g.path("Parent.vue")),
    );
    assert.equal(diagnostics.length, 0);
    const position = parent.indexOf("item.id", parent.indexOf("v-slot")) + "item.".length;
    const slotHover = query("generic-slot-hover", { position }, () =>
      g.languageService.getQuickInfoAtPosition(g.path("Parent.vue"), position),
    );
    assert.equal(ts.displayPartsToString(slotHover.displayParts), "(property) id: number");
    g.capture("generic-slots");
  } finally {
    g.languageService.dispose();
  }

  const aliasSource =
    '<script setup lang="ts">import { value } from "@lib/value"; const use = value;</script><template>{{ use }}</template>';
  const a = fixture(
    "resolver-options",
    { "Alias.vue": aliasSource, "src/value.ts": "export const value = 42;" },
    { paths: { "@lib/*": [path.join(workspace, "src/*")] } },
  );
  try {
    assert.equal(
      query("alias-diagnostics", {}, () =>
        a.languageService.getSemanticDiagnostics(a.path("Alias.vue")),
      ).length,
      0,
    );
    const position = aliasSource.indexOf("use = value") + "use = ".length;
    const definition = query("alias-definition", { position }, () =>
      a.languageService.getDefinitionAtPosition(a.path("Alias.vue"), position),
    );
    assert(definition.some((d) => d.fileName === a.path("src/value.ts")));
    a.capture("resolver-options");
  } finally {
    a.languageService.dispose();
  }

  assert.equal(createVizeLanguagePlugin(ts).getLanguageId("Story.art"), undefined);
  assert.equal(createVizeLanguagePlugin(ts).getLanguageId("Story.art.vue"), undefined);
  const art = fixture(
    "art-opt-in",
    {
      "Story.art":
        '<script setup lang="ts">const message = "story";</script><template>{{ message }}</template>',
    },
    {},
    { includeArt: true },
  );
  try {
    query("art-vue-form-diagnostics", {}, () =>
      art.languageService.getSemanticDiagnostics(art.path("Story.art")),
    );
    const position = art.text("Story.art").lastIndexOf("message");
    const hover = query("art-vue-form-hover", { position }, () =>
      art.languageService.getQuickInfoAtPosition(art.path("Story.art"), position),
    );
    assert(hover);
    art.capture("art-opt-in-vue-content");
  } finally {
    art.languageService.dispose();
  }

  const storySource =
    '<script setup lang="ts">\nimport { suffix } from "./art-lib";\nconst label: string = "😀";\nconst record = { alpha: 1, beta: "two" };\n</script>\n<art title="按钮"><variant name="Default"><button>{{ label.toUpperCase() }} {{ record.alpha }} {{ suffix }} {{ unknownStoryBinding }}</button></variant><variant name="Other"><span>{{ label }}</span></variant></art>';
  const story = fixture(
    "musea-variants",
    { "Story.art.vue": storySource, "art-lib.ts": 'export const suffix = "!";' },
    {},
    { includeArt: true },
  );
  try {
    const file = story.path("Story.art.vue");
    const diagnostics = query("musea-variant-diagnostics", { file }, () =>
      story.languageService.getSemanticDiagnostics(file),
    );
    assert.equal(diagnostics.length, 1);
    assert.equal(diagnostics[0].code, 2339);
    assert.equal(
      storySource.slice(diagnostics[0].start, diagnostics[0].start + diagnostics[0].length),
      "unknownStoryBinding",
    );
    const label = storySource.indexOf("label.toUpperCase");
    const hover = query("musea-variant-hover", { file, position: label }, () =>
      story.languageService.getQuickInfoAtPosition(file, label),
    );
    assert(ts.displayPartsToString(hover.displayParts).includes("string"));
    const property = storySource.indexOf("record.alpha") + "record.".length;
    const completion = query("musea-variant-completion", { file, position: property }, () =>
      story.languageService.getCompletionsAtPosition(file, property, {}),
    );
    assert.deepEqual(completion.entries.map((entry) => entry.name).sort(), ["alpha", "beta"]);
    const imported = storySource.lastIndexOf("suffix");
    const definition = query("musea-variant-definition", { file, position: imported }, () =>
      story.languageService.getDefinitionAtPosition(file, imported),
    );
    assert(definition.some((entry) => entry.fileName === story.path("art-lib.ts")));
    const references = query("musea-variant-references", { file, position: label }, () =>
      story.languageService.findReferences(file, label),
    );
    assert.equal(references.flatMap((entry) => entry.references).length, 3);
    const rename = query("musea-variant-rename", { file, position: label }, () =>
      story.languageService.findRenameLocations(file, label, false, false, true),
    );
    assert.equal(rename.length, 3);
    story.capture("native-art-variant-projection");
    story.update(
      "Story.art.vue",
      storySource.replace('const label: string = "😀"', "const label: number = 7"),
    );
    const updated = query("musea-unsaved-update-diagnostics", { file }, () =>
      story.languageService.getSemanticDiagnostics(file),
    );
    assert(updated.some((diagnostic) => diagnostic.code === 2339));
    story.delete("art-lib.ts");
    const deleted = query("musea-delete-import-diagnostics", { file }, () =>
      story.languageService.getSemanticDiagnostics(file),
    );
    assert(deleted.some((diagnostic) => diagnostic.code === 2307));
    story.capture("native-art-update-and-delete");
    story.update("Story.art.vue", '<art title="Empty"></art>');
    const empty = query("musea-no-variant-diagnostics", { file }, () =>
      story.languageService.getSemanticDiagnostics(file),
    );
    assert(empty.some((diagnostic) => diagnostic.code === 100003));
    story.capture("native-art-no-checkable-variant");
  } finally {
    story.languageService.dispose();
  }

  const javascript =
    "<script>const value = 1; value.missing();</script><template>{{ value }}</template>";
  for (const checkJavaScript of [false, true]) {
    const j = fixture(
      `javascript-${checkJavaScript}`,
      { "JavaScript.vue": javascript },
      {},
      { checkJavaScript },
    );
    try {
      const result = query(`javascript-check-${checkJavaScript}`, {}, () =>
        j.languageService.getSemanticDiagnostics(j.path("JavaScript.vue")),
      );
      assert.equal(
        result.some((diagnostic) => diagnostic.code === 2339),
        checkJavaScript,
      );
      j.capture("javascript-opt-in");
    } finally {
      j.languageService.dispose();
    }
  }

  const eventChild =
    '<script setup lang="ts">defineProps<{ fooBar: string }>(); defineEmits<{ saveItem: [value: string] }>();</script><template />';
  const eventParent =
    '<script setup lang="ts">import Child from "./Child.vue";</script><template><Child :foo-bar="1" @save-item="(value: number) => undefined" /></template>';
  const e = fixture("feature-boundaries", { "Child.vue": eventChild, "Events.vue": eventParent });
  try {
    const file = e.path("Events.vue");
    const errors = query("feature-boundary-diagnostics", { file }, () =>
      e.languageService.getSemanticDiagnostics(file),
    );
    assert.equal(
      errors.filter((diagnostic) => diagnostic.code === 2322).length,
      2,
      "both the wrong prop and wrong event parameter survive; native prop echo stays suppressed",
    );
    assert(
      errors.some((diagnostic) =>
        ts
          .flattenDiagnosticMessageText(diagnostic.messageText, "\n")
          .includes("Types of parameters 'value'"),
      ),
    );
    const root = e.language.scripts.get(file).generated.root;
    const diagnosticOnly = root.projection.mappings
      .map((row, index) => ({ row, mapping: root.embeddedCodes[0].mappings[index] }))
      .filter(({ row }) => row[5] === 0);
    assert(diagnosticOnly.length > 0, "native event handler provides diagnostic-only anchors");
    for (const { mapping } of diagnosticOnly) {
      assert.equal(mapping.data.navigation, false);
      assert.equal(mapping.data.completion, false);
      assert.equal(mapping.data.semantic, false);
    }
    const position = eventParent.indexOf("foo-bar");
    const rename = query("rewritten-prop-refuses-rename", { file, position }, () =>
      e.languageService.getRenameInfo(file, position),
    );
    assert.equal(rename.canRename, false);
    e.capture("diagnostic-only-and-rewritten-alias");
  } finally {
    e.languageService.dispose();
  }
});

void test("authored source files share work per snapshot without sharing file identities", () => {
  const source =
    '<script setup lang="ts">const value: number = "bad";</script><template>{{ value }}</template>';
  const initial = ts.ScriptSnapshot.fromString(source);
  const snapshots = new Map([
    ["A.vue", initial],
    ["B.vue", initial],
  ]);
  const parsed = [];
  const f = fixture(
    "source-file-cache",
    { "A.vue": source, "B.vue": source },
    {},
    {},
    {
      typescript: {
        ...ts,
        createSourceFile(file, ...args) {
          if (file.endsWith(".vue")) parsed.push(file);
          return ts.createSourceFile(file, ...args);
        },
      },
      host: {
        getScriptSnapshot(file) {
          const snapshot = snapshots.get(path.basename(file));
          if (snapshot) return snapshot;
          const text = ts.sys.readFile(file);
          return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
        },
      },
    },
  );
  try {
    for (const name of ["A.vue", "B.vue", "A.vue", "B.vue"]) {
      const file = f.path(name);
      const diagnostics = f.languageService.getSemanticDiagnostics(file);
      assert(diagnostics.some((d) => d.code === 2322 && d.file.fileName === file));
      assert.deepEqual(f.languageService.toLineColumnOffset(file, 0), { line: 0, character: 0 });
    }
    assert.deepEqual(parsed, [f.path("A.vue"), f.path("B.vue")]);
    const updated = source.replace('"bad"', '"still bad"');
    snapshots.set("A.vue", ts.ScriptSnapshot.fromString(updated));
    f.update("A.vue", updated);
    const diagnostics = f.languageService.getSemanticDiagnostics(f.path("A.vue"));
    assert(diagnostics.some((d) => d.code === 2322 && d.file.text === updated));
    f.languageService.toLineColumnOffset(f.path("B.vue"), 0);
    assert.equal(parsed.filter((file) => file === f.path("A.vue")).length, 2);
    assert.equal(parsed.filter((file) => file === f.path("B.vue")).length, 1);
  } finally {
    f.languageService.dispose();
  }
});

void test("the host's legacy module resolver keeps authority over virtual modules", () => {
  const requested = [];
  const f = fixture(
    "host-resolver",
    {
      "main.ts": "import { value } from '#virtual'; const result: number = value;",
      "implementation.ts": "export const value = 42;",
    },
    {},
    {},
    {
      host: {
        resolveModuleNames(names) {
          requested.push(...names);
          return names.map((name) =>
            name === "#virtual"
              ? {
                  resolvedFileName: path.join(workspace, "implementation.ts"),
                  extension: ts.Extension.Ts,
                  isExternalLibraryImport: false,
                }
              : undefined,
          );
        },
      },
    },
  );
  try {
    assert.deepEqual(f.languageService.getSemanticDiagnostics(f.path("main.ts")), []);
    assert(requested.includes("#virtual"));
  } finally {
    f.languageService.dispose();
  }
});

void test("all captured packets equal the reviewed complete source expectations", () => {
  fs.mkdirSync(evidence, { recursive: true });
  fs.writeFileSync(path.join(evidence, "packets.json"), JSON.stringify(packets, null, 2) + "\n");
  fs.writeFileSync(
    path.join(evidence, "inputs.json"),
    JSON.stringify(normalize(inputs), null, 2) + "\n",
  );
  fs.writeFileSync(
    path.join(evidence, "measurements.json"),
    JSON.stringify(
      {
        typescript: ts.version,
        platform: process.platform,
        arch: process.arch,
        node: process.version,
        rows: timings,
        packetsSha256: hash(JSON.stringify(packets)),
      },
      null,
      2,
    ) + "\n",
  );
  const expectations = path.join(directory, "expected-packets");
  const expected = [
    "basic.json",
    "updates-and-language.json",
    "art.json",
    "feature-boundaries.json",
  ].flatMap((file) => JSON.parse(fs.readFileSync(path.join(expectations, file), "utf8")));
  const provenance = JSON.parse(
    fs.readFileSync(path.join(directory, "expected-packets.provenance.json"), "utf8"),
  );
  assert.equal(hash(JSON.stringify(expected)), provenance.expectedPacketDataSha256);
  assert.equal(expected.length, provenance.expectedPackets);
  assert.deepEqual(packets, publicInstanceReference(expected));
});
