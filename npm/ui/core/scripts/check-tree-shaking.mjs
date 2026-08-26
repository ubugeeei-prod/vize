import assert from "node:assert/strict";
import path from "node:path";
import { gzipSync } from "node:zlib";

import { build } from "vite";

import { uiFamilyCatalog } from "../src/family-catalog.ts";

const virtualConsumerId = path.resolve(".vize-ui-tree-shaking-consumer.mjs");

/**
 * Bundle an in-package consumer through the public package exports.
 *
 * Resolving the virtual entry inside this package intentionally exercises Node
 * package self-references, conditional exports, sideEffects metadata, Rolldown,
 * CSS extraction, and production minification together.
 */
async function bundleConsumer(source) {
  const result = await build({
    configFile: false,
    logLevel: "silent",
    plugins: [
      {
        name: "vize-ui-tree-shaking-consumer",
        resolveId(id) {
          if (id === "virtual:vize-ui-consumer" || id === virtualConsumerId) {
            return virtualConsumerId;
          }
        },
        load(id) {
          if (id === virtualConsumerId) return source;
        },
      },
    ],
    build: {
      cssCodeSplit: true,
      minify: true,
      target: "es2022",
      write: false,
      rollupOptions: {
        input: "virtual:vize-ui-consumer",
        external: (id) => id === "vue" || id.startsWith("vue/"),
      },
    },
  });

  assert.ok(!Array.isArray(result), "tree-shaking build unexpectedly returned multiple outputs");
  const javascript = result.output
    .filter((output) => output.type === "chunk")
    .map((output) => output.code)
    .join("\n");
  const css = result.output
    .filter((output) => output.type === "asset" && output.fileName.endsWith(".css"))
    .map((output) => String(output.source))
    .join("\n");

  return Object.freeze({
    javascript,
    css,
    javascriptBytes: Buffer.byteLength(javascript),
    javascriptGzipBytes: gzipSync(javascript).byteLength,
    cssBytes: Buffer.byteLength(css),
    cssGzipBytes: css.length === 0 ? 0 : gzipSync(css).byteLength,
  });
}

/** Keep a selected component observable so the entry export cannot be discarded. */
function consumerSource(exportName, packageEntry) {
  return `import { ${exportName} } from ${JSON.stringify(packageEntry)};globalThis.__vizeUiConsumer=${exportName};`;
}

const treeShakingEntries = uiFamilyCatalog.filter((entry) => entry.bundleBudget);
const familySignatures = new Map(
  treeShakingEntries.map((entry) => [
    entry.canonicalName,
    new RegExp(entry.bundleBudget.retainedSignature),
  ]),
);

for (const { canonicalName: family, bundleBudget } of treeShakingEntries) {
  const {
    exportName,
    allowedRetainedFamilies = [],
    maximumJavaScriptGzipBytes,
    maximumCssGzipBytes,
  } = bundleBudget;
  const rootOutput = await bundleConsumer(consumerSource(exportName, "@vizejs/ui"));
  const subpathOutput = await bundleConsumer(consumerSource(exportName, `@vizejs/ui/${family}`));
  assert.equal(
    rootOutput.javascript,
    subpathOutput.javascript,
    `${exportName} root and subpath exports emitted different JavaScript`,
  );
  assert.equal(
    rootOutput.css,
    subpathOutput.css,
    `${exportName} root and subpath exports emitted different CSS`,
  );

  for (const [signatureFamily, signature] of familySignatures) {
    const allowed = new Set([family, ...allowedRetainedFamilies]);
    if (allowed.has(signatureFamily)) {
      assert.match(
        rootOutput.javascript,
        signature,
        `${exportName} was eliminated from its own consumer bundle`,
      );
    } else {
      assert.doesNotMatch(
        rootOutput.javascript,
        signature,
        `${exportName} retained the unused ${signatureFamily} family`,
      );
    }
  }

  // Styled families pull the shared packaged stylesheet (dist has exactly one
  // style.css); unstyled families must not retain any CSS at all.
  const styledFamilySignatures = new Map([
    ["motion", /--vize-ui-motion-duration-fast/],
    ["visually-hidden", /clip-path:inset\(50%\)/],
  ]);
  const styledSignature = styledFamilySignatures.get(family);
  if (styledSignature) {
    assert.match(rootOutput.css, styledSignature);
  } else {
    assert.equal(rootOutput.css, "", `${exportName} retained another component's stylesheet`);
  }

  for (const { entry, output } of [
    { entry: "root", output: rootOutput },
    { entry: "subpath", output: subpathOutput },
  ]) {
    console.log(
      JSON.stringify({
        check: "@vizejs/ui consumer tree shaking",
        entry: `${entry}-${family}`,
        javascriptBytes: output.javascriptBytes,
        javascriptGzipBytes: output.javascriptGzipBytes,
        maximumJavaScriptGzipBytes,
        cssBytes: output.cssBytes,
        cssGzipBytes: output.cssGzipBytes,
        maximumCssGzipBytes,
      }),
    );

    assert.ok(
      output.javascriptGzipBytes <= maximumJavaScriptGzipBytes,
      `${entry}-${family} JavaScript is ${output.javascriptGzipBytes} gzip bytes; budget is ${maximumJavaScriptGzipBytes}`,
    );
    assert.ok(
      output.cssGzipBytes <= maximumCssGzipBytes,
      `${entry}-${family} CSS is ${output.cssGzipBytes} gzip bytes; budget is ${maximumCssGzipBytes}`,
    );
  }
}
