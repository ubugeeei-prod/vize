import vize from "@vizejs/unplugin/esbuild";
import { build } from "esbuild";

import {
  exampleRoot,
  findStyleAsset,
  isDirectRun,
  prepareOutputDirectory,
  writePreviewHtml,
} from "./shared.ts";

export async function buildEsbuild(outputRoot?: string): Promise<string> {
  const outputDirectory = prepareOutputDirectory("esbuild", outputRoot);
  await build({
    absWorkingDir: exampleRoot,
    entryPoints: ["src/main.js"],
    bundle: true,
    format: "esm",
    minify: true,
    outdir: outputDirectory,
    entryNames: "app",
    assetNames: "[name]",
    plugins: [vize({ isProduction: true, root: exampleRoot })],
  });
  writePreviewHtml("esbuild", outputDirectory, "app.js", findStyleAsset(outputDirectory));
  return outputDirectory;
}

if (isDirectRun(import.meta.url)) await buildEsbuild();
