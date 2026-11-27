import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL("../", import.meta.url));
const root = resolve(directory, "../../..");
const temporary = mkdtempSync(join(tmpdir(), "vize-swc-packed-"));
const native = join(temporary, "native");
const plugin = join(temporary, "plugin");

function run(command: string, args: string[], cwd: string, capture = false): string {
  const result = spawnSync(command, args, {
    cwd,
    encoding: "utf8",
    stdio: capture ? ["ignore", "pipe", "pipe"] : "inherit",
    env: { ...process.env, VIZE_SWC_CONSUMER: temporary },
  });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed (${result.status})\n${result.stderr ?? ""}`);
  return result.stdout ?? "";
}

cpSync(join(directory, "dist"), join(plugin, "dist"), { recursive: true });
cpSync(join(directory, "README.md"), join(plugin, "README.md"));
const manifest = JSON.parse(readFileSync(join(directory, "package.json"), "utf8"));
// A published native version is useful for first-publish bootstrap verification.
// CI normally exercises a tarball of the exact native source built in this job.
const version = process.env.VIZE_SWC_NATIVE_VERSION ?? manifest.version;
manifest.version = version;
manifest.dependencies["@vizejs/native"] = version;
manifest.dependencies["oxc-parser"] = JSON.parse(
  readFileSync(join(directory, "node_modules/oxc-parser/package.json"), "utf8"),
).version;
delete manifest.devDependencies;
delete manifest.scripts;
writeFileSync(join(plugin, "package.json"), JSON.stringify(manifest, null, 2));

const dependencies: Record<string, string> = {
  "@swc/core": "1.16.22",
  vue: "3.5.35",
  "@vue/server-renderer": "3.5.35",
};
const overrides: Record<string, string> = {};
if (!process.env.VIZE_SWC_NATIVE_VERSION) {
  const nativeRoot = join(root, "npm/native");
  const nativeManifest = JSON.parse(readFileSync(join(nativeRoot, "package.json"), "utf8"));
  cpSync(join(nativeRoot, "package.json"), join(native, "package.json"), { recursive: true });
  const files = readdirSync(nativeRoot).filter((file) => /\.(?:js|node)$|\.d\.ts$/.test(file));
  if (!files.some((file) => file.endsWith(".node")))
    throw new Error("Build the source native binding before packed SWC tests");
  for (const file of files) cpSync(join(nativeRoot, file), join(native, file));
  nativeManifest.files = files;
  delete nativeManifest.scripts;
  delete nativeManifest.devDependencies;
  delete nativeManifest.optionalDependencies;
  writeFileSync(join(native, "package.json"), JSON.stringify(nativeManifest, null, 2));
  const packed = JSON.parse(run("npm", ["pack", "--ignore-scripts", "--json"], native, true));
  overrides["@vizejs/native"] = `file:${join(native, packed[0].filename)}`;
}
const packed = JSON.parse(run("npm", ["pack", "--ignore-scripts", "--json"], plugin, true));
dependencies[manifest.name] = `file:${join(plugin, packed[0].filename)}`;
writeFileSync(
  join(temporary, "package.json"),
  JSON.stringify({ private: true, dependencies, overrides }, null, 2),
);
run("npm", ["install", "--ignore-scripts", "--no-audit", "--no-fund"], temporary);
writeFileSync(
  join(temporary, ".swcrc"),
  JSON.stringify({
    jsc: {
      parser: { syntax: "typescript", tsx: true },
      experimental: {
        runPluginFirst: true,
        plugins: [["@vizejs/swc-plugin", {}]],
      },
    },
    module: { type: "commonjs" },
  }),
);
writeFileSync(join(temporary, "App.tsx"), "export const App = () => <div>swcrc</div>");
run(
  process.execPath,
  [
    "-e",
    `const s = require('@swc/core'); const r = s.transformFileSync('App.tsx'); if (!r.code.includes('createElementBlock')) throw new Error('Named .swcrc plugin did not compile JSX');`,
  ],
  temporary,
);
run(process.execPath, ["--test", join(directory, "test/contracts.test.ts")], temporary);
console.log(`Verified packed consumer: ${temporary}`);
console.log(`Reviewable publish tarball: ${join(plugin, packed[0].filename)}`);
