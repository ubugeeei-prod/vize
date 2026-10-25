import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { startTsserver } from "./tsserver-client.mjs";

export function createInheritedConfig(consumer, src, output, config) {
  const base = path.join(consumer, "inherited.json"),
    root = path.join(consumer, "tsconfig.json");
  const sourceFile = path.join(src, "Inherited.vue");
  const source =
    '<script setup lang="ts">\nconst known = 1;\n</script>\n<template><UnknownClosureProbe :n="known"/></template>\n';
  const control = path.join(src, "inherited-control.ts");
  const controlSource =
    'export function fallthrough(kind: string) {\n  switch (kind) {\n    case "one": kind.trim();\n    case "two": break;\n  }\n}\n';
  const before = {
    compilerOptions: { noFallthroughCasesInSwitch: false },
    vueCompilerOptions: { strictTemplates: false },
  };
  const after = {
    compilerOptions: { noFallthroughCasesInSwitch: true },
    vueCompilerOptions: { strictTemplates: true },
  };
  fs.writeFileSync(root, JSON.stringify({ ...config, extends: "./inherited.json" }));
  fs.writeFileSync(base, JSON.stringify(before));
  fs.writeFileSync(sourceFile, source);
  fs.writeFileSync(control, controlSource);
  const digest = (data) => createHash("sha256").update(data).digest("hex");
  const fixed = [root, sourceFile, control].map((file) => ({
    file,
    bytes: fs.readFileSync(file),
    sha256: digest(fs.readFileSync(file)),
  }));
  fs.writeFileSync(
    path.join(output, "inherited-inputs.json"),
    JSON.stringify(
      {
        root: fs.readFileSync(root, "utf8"),
        source,
        controlSource,
        before,
        after,
        fixed: fixed.map(({ file, sha256 }) => ({ file, sha256 })),
      },
      null,
      2,
    ),
  );
  return { root, base, sourceFile, source, control, controlSource, before, after, fixed };
}

export async function checkInheritedConfig(
  client,
  fixture,
  native,
  tsserver,
  consumer,
  output,
  results,
) {
  const packet = {
    source: fixture.source,
    beforeProjection: null,
    afterProjection: null,
    before: null,
    after: null,
    fresh: null,
    controlBefore: null,
    controlAfter: null,
  };
  const save = () =>
    fs.writeFileSync(
      path.join(output, "inherited-observations.json"),
      JSON.stringify(packet, null, 2),
    );
  const project = (source) =>
    native.generateVueLanguageCode(source, {
      filename: fixture.sourceFile,
      configFileName: fixture.root,
      checkJavaScript: false,
      preserveUnusedDiagnostics: true,
    });
  const request = async (server, key, command, file) => {
    const response = await server.send(command, { file });
    packet[key] = response;
    results["inherited-" + key] = response;
    save();
    assert.equal(response.success, true, JSON.stringify(response));
    return response;
  };
  await client.send(
    "open",
    { file: fixture.sourceFile, fileContent: fixture.source, scriptKindName: "TS" },
    false,
  );
  await client.send("open", { file: fixture.control, fileContent: fixture.controlSource }, false);
  packet.beforeProjection = JSON.parse(project(fixture.source));
  save();
  await request(client, "before", "semanticDiagnosticsSync", fixture.sourceFile);
  await request(client, "controlBefore", "semanticDiagnosticsSync", fixture.control);
  fs.writeFileSync(fixture.base, JSON.stringify(fixture.after));
  for (const file of fixture.fixed)
    assert.deepEqual(fs.readFileSync(file.file), file.bytes, "only the inherited config changes");
  packet.afterProjection = JSON.parse(project(fixture.source));
  save();
  await client.send("reloadProjects", {}, false);
  await request(client, "controlAfter", "semanticDiagnosticsSync", fixture.control);
  await request(client, "after", "semanticDiagnosticsSync", fixture.sourceFile);
  const freshDirectory = path.join(output, "inherited-fresh-server");
  fs.mkdirSync(freshDirectory);
  const fresh = startTsserver(tsserver, consumer, freshDirectory);
  try {
    await fresh.send("configure", {
      extraFileExtensions: [
        { extension: ".vue", isMixedContent: true, scriptKind: 7 },
        { extension: ".art", isMixedContent: true, scriptKind: 7 },
      ],
    });
    await fresh.send(
      "open",
      { file: fixture.sourceFile, fileContent: fixture.source, scriptKindName: "TS" },
      false,
    );
    await request(fresh, "fresh", "semanticDiagnosticsSync", fixture.sourceFile);
  } finally {
    await fresh.stop();
    save();
  }
  assert.deepEqual(packet.before.body, [], "unchecked unknown components are initially admitted");
  assert.deepEqual(packet.controlBefore.body, []);
  assert(
    packet.controlAfter.body.some((diagnostic) => diagnostic.code === 7029),
    "the real TypeScript project consumed the changed extends file",
  );
  assert.notEqual(
    packet.beforeProjection.text,
    packet.afterProjection.text,
    "the native producer independently consumes effective inherited Vue options",
  );
  assert(
    packet.fresh.body.length > 0,
    "a fresh actual tsserver enforces the changed strict template option",
  );
  assert.deepEqual(
    packet.after.body,
    packet.fresh.body,
    "the warm server must refresh the same full Vue diagnostic packet after project reload",
  );
}
