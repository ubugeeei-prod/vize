import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { patchUnoCssBridge } from "./unocss.ts";

const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-unocss-"));
const sourcePath = path.join(tempRoot, "App.vue");
const virtualId = `\0${sourcePath}.ts`;
const plainBuildId = `${sourcePath}.ts`;
const queriedClientVirtualId = `${virtualId}?macro=true`;
const queriedSsrVirtualId = `\0vize-ssr:${sourcePath}.ts?vue&type=template`;
const queriedPlainSsrBuildId = `vize-ssr:${sourcePath}.ts?vue&type=template`;

fs.writeFileSync(
  sourcePath,
  `<template><div flex="~ col gap-2" text="sm slate-700">hello</div></template>\n`,
  "utf-8",
);

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", virtualId);

  assert.equal(receivedId, sourcePath);
  assert.match(receivedCode, /export default \{\}/);
  assert.match(receivedCode, /flex="~ col gap-2"/);
  assert.match(receivedCode, /text="sm slate-700"/);
}

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:transformers",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", virtualId);

  assert.equal(receivedId, sourcePath);
  assert.equal(receivedCode, "export default {}");
}

{
  let callCount = 0;

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform() {
        callCount += 1;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", virtualId);

  assert.equal(callCount, 1);
}

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:global:dev:scan",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", queriedClientVirtualId);

  assert.equal(receivedId, `${sourcePath}?macro=true`);
  assert.match(receivedCode, /export default \{\}/);
  assert.match(receivedCode, /flex="~ col gap-2"/);
}

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", queriedSsrVirtualId);

  assert.equal(receivedId, `${sourcePath}?vue&type=template`);
  assert.match(receivedCode, /export default \{\}/);
  assert.match(receivedCode, /text="sm slate-700"/);
}

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", plainBuildId);

  assert.equal(receivedId, sourcePath);
  assert.match(receivedCode, /export default \{\}/);
  assert.match(receivedCode, /flex="~ col gap-2"/);
}

{
  const scannedIds: string[] = [];
  const extractedAttributes: string[] = [];

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform(code: string, id: string) {
        if (!/\.(vue|svelte|[jt]sx|mdx?|astro|html)($|\?)/.test(id)) {
          return null;
        }

        scannedIds.push(id);
        extractedAttributes.push(
          ...Array.from(code.matchAll(/\b(?:flex|text)="([^"]+)"/g), ([, value]) => value!),
        );
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", plainBuildId);

  assert.deepEqual(scannedIds, [sourcePath]);
  assert.deepEqual(extractedAttributes, ["~ col gap-2", "sm slate-700"]);
}

{
  let receivedCode = "";
  let receivedId = "";

  const plugins = [
    {
      name: "unocss:global:build:scan",
      transform(code: string, id: string) {
        receivedCode = code;
        receivedId = id;
        return null;
      },
    },
  ];

  patchUnoCssBridge(plugins);
  plugins[0]!.transform!("export default {}", queriedPlainSsrBuildId);

  assert.equal(receivedId, `${sourcePath}?vue&type=template`);
  assert.match(receivedCode, /export default \{\}/);
  assert.match(receivedCode, /text="sm slate-700"/);
}

console.log("✅ vite-plugin-vize UnoCSS bridge tests passed!");
