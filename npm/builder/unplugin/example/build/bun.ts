import path from "node:path";

import vize from "@vizejs/unplugin/bun";

import { exampleRoot, findStyleAsset, prepareOutputDirectory, writePreviewHtml } from "./shared.ts";

const outputDirectory = prepareOutputDirectory("bun", process.argv[2]);
const result = await Bun.build({
  entrypoints: [path.join(exampleRoot, "src/main.js")],
  outdir: outputDirectory,
  target: "browser",
  format: "esm",
  minify: true,
  naming: "app.[ext]",
  plugins: [vize({ isProduction: true, root: exampleRoot })],
});

if (!result.success) throw new AggregateError(result.logs, "Bun could not build the Vize example");
writePreviewHtml("bun", outputDirectory, "app.js", findStyleAsset(outputDirectory));
