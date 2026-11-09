import vize from "@vizejs/unplugin/webpack";
import webpack from "webpack";

import {
  exampleRoot,
  findStyleAsset,
  isDirectRun,
  prepareOutputDirectory,
  writePreviewHtml,
} from "./shared.ts";

function runCompiler(compiler: webpack.Compiler): Promise<webpack.Stats> {
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close((closeError) => {
        if (error) reject(error);
        else if (closeError) reject(closeError);
        else if (stats === undefined) reject(new Error("webpack returned no build statistics"));
        else resolve(stats);
      });
    });
  });
}

export async function buildWebpack(outputRoot?: string): Promise<string> {
  const outputDirectory = prepareOutputDirectory("webpack", outputRoot);
  const compiler = webpack({
    context: exampleRoot,
    mode: "production",
    target: ["web", "es2022"],
    entry: "./src/main.js",
    output: {
      clean: true,
      filename: "app.js",
      path: outputDirectory,
    },
    experiments: { css: true },
    module: {
      rules: [{ test: /\.css$/u, type: "css/auto" }],
    },
    plugins: [vize({ isProduction: true, root: exampleRoot })],
    resolve: { extensions: [".js", ".ts", ".vue"] },
  });
  const stats = await runCompiler(compiler);
  if (stats.hasErrors()) {
    throw new Error(stats.toString({ all: false, errors: true, errorDetails: true }));
  }
  writePreviewHtml("webpack", outputDirectory, "app.js", findStyleAsset(outputDirectory));
  return outputDirectory;
}

if (isDirectRun(import.meta.url)) await buildWebpack();
