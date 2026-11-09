import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const exampleRoot = path.resolve(import.meta.dirname, "..");
export const defaultOutputRoot = path.join(exampleRoot, "dist");

export function prepareOutputDirectory(builder: string, outputRoot = defaultOutputRoot): string {
  const outputDirectory = path.join(outputRoot, builder);
  fs.rmSync(outputDirectory, { recursive: true, force: true });
  fs.mkdirSync(outputDirectory, { recursive: true });
  return outputDirectory;
}

export function writePreviewHtml(
  builder: string,
  outputDirectory: string,
  scriptFile = "app.js",
  styleFile?: string,
): void {
  const stylesheet =
    styleFile === undefined ? "" : `    <link rel="stylesheet" href="./${styleFile}" />\n`;
  fs.writeFileSync(
    path.join(outputDirectory, "index.html"),
    `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="vize-builder" content="${builder}" />
    <title>Vize ${builder} example</title>
${stylesheet}  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="./${scriptFile}"></script>
  </body>
</html>
`,
  );
}

export function findStyleAsset(outputDirectory: string): string | undefined {
  return fs.readdirSync(outputDirectory).find((file) => file.endsWith(".css"));
}

export function isDirectRun(moduleUrl: string): boolean {
  return (
    process.argv[1] !== undefined && path.resolve(process.argv[1]) === fileURLToPath(moduleUrl)
  );
}
