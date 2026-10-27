import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { position } from "./tsserver-client.mjs";
import type { TsserverClient } from "./project-metadata.ts";

type CallItem = Record<"from" | "to", { file: string; name: string }>;

export function createNavigationFixtures(src: string, output: string) {
  const directory = path.join(src, "navigation");
  fs.mkdirSync(directory);
  const importer = path.join(directory, "imports.ts");
  const one = path.join(directory, "One.vue"),
    two = path.join(directory, "Two.vue");
  const source = [
    'const compass = "😀 日本語";',
    'import Card from "./One.vue";',
    'import { default as Named } from "./One.vue";',
    'import { Bridge } from "./bridge";',
    "Card; Named; Bridge;",
    "function shadow(Card: number) { return Card; }",
    "const object = { Card: 1 }; object.Card;",
    "const shorthand = { Card };",
    "",
  ].join("\r\n");
  const calls = path.join(directory, "calls.ts"),
    caller = path.join(directory, "Caller.vue");
  const callsSource =
    "export function shared() { return 1; }\nexport function plain() { return shared(); }\n";
  const callerSource =
    '<script setup lang="ts">\nimport { shared } from "./calls";\nfunction caller() { return shared(); }\nconst result = caller();\n</script>\n<template>{{ result }}</template>\n';
  const inputs = [
    [importer, source],
    [
      one,
      '<script setup lang="ts">\r\ndefineProps<{ one: string }>();\r\n</script>\r\n<template><div/></template>\r\n',
    ],
    [
      two,
      '<script setup lang="ts">\r\ndefineProps<{ two: number }>();\r\n</script>\r\n<template><div/></template>\r\n',
    ],
    [path.join(directory, "bridge.ts"), 'export { default as Bridge } from "./One.vue";\n'],
    [calls, callsSource],
    [caller, callerSource],
  ];
  for (const [file, source] of inputs) fs.writeFileSync(file, source);
  fs.writeFileSync(
    path.join(output, "navigation-inputs.json"),
    JSON.stringify(
      inputs.map(([file, source]) => ({
        file,
        source,
        sha256: createHash("sha256").update(source).digest("hex"),
      })),
      null,
      2,
    ),
  );
  return { importer, one, two, source, calls, callsSource, caller, callerSource };
}

export async function checkNavigation(
  client: TsserverClient,
  fixture: ReturnType<typeof createNavigationFixtures>,
  results: Record<string, unknown>,
  vue: string,
  vueText: string,
  util: string,
) {
  const { importer, one, two, source, calls, callsSource, caller, callerSource } = fixture;
  const request = async (
    name: string,
    command: string,
    file: string,
    text?: string,
    marker?: string,
    delta = 1,
  ) => {
    const response = await client.send(command, {
      file,
      ...(marker ? position(text, marker, delta) : {}),
    });
    results[name] = response;
    assert.equal(response.success, true, JSON.stringify(response));
    return response;
  };
  const change = (file: string, old: string, text: string) => {
    const lines = old.split("\n");
    return client.send("change", {
      file,
      line: 1,
      offset: 1,
      endLine: lines.length,
      endOffset: (lines.at(-1) ?? "").length + 1,
      insertString: text,
    });
  };
  await client.send("open", { file: importer, fileContent: source }, false);
  for (const [name, marker] of [
    ["default", "Card;"],
    ["named-default", "Named;"],
    ["barrel", "Bridge;"],
  ]) {
    const definition = await request("navigation-" + name, "definition", importer, source, marker);
    assert.equal(definition.body[0].file, one);
    assert.deepEqual(definition.body[0].start, { line: 1, offset: 1 });
  }
  const opened = source.replace('Card from "./One.vue"', 'Card from "./Two.vue"');
  await change(importer, source, opened);
  assert.equal(fs.readFileSync(importer, "utf8"), source, "the import edit remains unsaved");
  const changedDefinition = await request(
    "navigation-unsaved-import",
    "definition",
    importer,
    opened,
    "Card;",
  );
  assert.equal(changedDefinition.body[0].file, two);
  const changedHover = await request(
    "navigation-unsaved-hover",
    "quickinfo",
    importer,
    opened,
    "Card;",
  );
  assert.match(changedHover.body.displayString, /two: number/);
  assert.doesNotMatch(changedHover.body.displayString, /one: string/);
  const parameter = await request(
    "navigation-shadow-parameter",
    "definition",
    importer,
    opened,
    "return Card",
    "return ".length + 1,
  );
  assert.equal(parameter.body[0].file, importer);
  assert.deepEqual(parameter.body[0].start, position(opened, "Card: number", 0));
  const parameterHover = await request(
    "navigation-shadow-hover",
    "quickinfo",
    importer,
    opened,
    "return Card",
    "return ".length + 1,
  );
  assert.match(parameterHover.body.displayString, /Card: number/);
  assert.doesNotMatch(parameterHover.body.displayString, /VueComponent/);
  const propertyHover = await request(
    "navigation-property-name",
    "quickinfo",
    importer,
    opened,
    "object.Card",
    "object.".length + 1,
  );
  assert.match(propertyHover.body.displayString, /Card: number/);
  assert.doesNotMatch(propertyHover.body.displayString, /VueComponent/);
  const shorthand = await request(
    "navigation-shorthand-value",
    "definition",
    importer,
    opened,
    "{ Card }",
    3,
  );
  assert.equal(shorthand.body[0].file, two);
  await change(importer, opened, source);
  await client.send("close", { file: importer }, false);
  assert.equal(fs.readFileSync(importer, "utf8"), source);

  await client.send("open", { file: util, fileContent: fs.readFileSync(util, "utf8") }, false);
  const fileReferences = await request("authored-file-references", "fileReferences", util);
  const authored = fileReferences.body.refs.find((entry: { file: string }) => entry.file === vue);
  assert(authored, JSON.stringify(fileReferences));
  assert.deepEqual(authored.start, position(vueText, "./util", 0));
  assert.deepEqual(authored.end, position(vueText, "./util", "./util".length));
  assert.equal(authored.lineText, vueText.split("\n")[1]);

  await client.send("open", { file: calls, fileContent: callsSource }, false);
  await client.send(
    "open",
    { file: caller, fileContent: callerSource, scriptKindName: "TS" },
    false,
  );
  const incoming = await request(
    "safe-incoming-call-hierarchy",
    "provideCallHierarchyIncomingCalls",
    calls,
    callsSource,
    "shared",
  );
  assert(
    incoming.body.some((call: CallItem) => call.from.file === calls && call.from.name === "plain"),
    JSON.stringify(incoming),
  );
  assert(
    incoming.body.every((call: CallItem) => call.from.file !== caller),
    JSON.stringify(incoming),
  );
  const outgoing = await request(
    "ordinary-outgoing-call-hierarchy",
    "provideCallHierarchyOutgoingCalls",
    calls,
    callsSource,
    "plain",
  );
  assert(
    outgoing.body.some((call: CallItem) => call.to.file === calls && call.to.name === "shared"),
    JSON.stringify(outgoing),
  );
  const refused = await request(
    "unsupported-vue-outgoing-call-hierarchy",
    "provideCallHierarchyOutgoingCalls",
    caller,
    callerSource,
    "caller()",
  );
  assert.deepEqual(refused.body, []);
}
