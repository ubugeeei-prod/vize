import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const readJson = (file) => JSON.parse(readFileSync(file, "utf8"));
const entrypoints = {
  "mod.ts": ["npm/cli", "vize", false],
  "config.ts": ["npm/cli", "vize", false, "/config"],
  "native.ts": ["npm/native", "@vizejs/native", false],
  "vite.ts": ["npm/builder/vite", "@vizejs/vite-plugin", true],
};

export function preparePackage(output, { root = repository, version } = {}) {
  const current = readJson(resolve(root, "npm/cli/package.json")).version;
  version ??= current;
  if (!/^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version) || version !== current) {
    throw new Error(`JSR version must match the checked-out Vize release (${current})`);
  }
  if (!output || resolve(output) === resolve(root, "jsr/vize")) {
    throw new Error("Generate the JSR package in a separate staging directory");
  }
  const manifest = readJson(resolve(root, "jsr/vize/jsr.json"));
  for (const [directory] of Object.values(entrypoints)) {
    if (readJson(resolve(root, directory, "package.json")).version !== version) {
      throw new Error(`${directory} must match JSR version ${version}`);
    }
  }
  mkdirSync(output, { recursive: true });
  // Publication immediately follows the exact npm release; Deno's default
  // dependency age would otherwise reject those freshly published versions.
  writeFileSync(
    resolve(output, "jsr.json"),
    `${JSON.stringify({ ...manifest, version, minimumDependencyAge: 0 }, null, 2)}\n`,
  );
  for (const [filename, [, name, hasDefault, subpath = ""]] of Object.entries(entrypoints)) {
    const specifier = `npm:${name}@${version}${subpath}`;
    const source = `/** Node.js 22+ entry point for ${name}. */\nexport * from ${JSON.stringify(specifier)};\n`;
    writeFileSync(
      resolve(output, filename),
      source + (hasDefault ? `export { default } from ${JSON.stringify(specifier)};\n` : ""),
    );
  }
  for (const [source, target] of [
    ["jsr/vize/README.md", "README.md"],
    ["LICENSE", "LICENSE"],
  ]) {
    writeFileSync(resolve(output, target), readFileSync(resolve(root, source)));
  }
  return version;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [output, version] = process.argv.slice(2);
  console.log(preparePackage(output, { version: version?.replace(/^v/, "") }));
}
