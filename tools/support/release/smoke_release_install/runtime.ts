import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { constants } from "node:os";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { readPackageJson, resolveInstalledBin, type PackageInfo } from "./manifest.ts";
import {
  assertExecutable,
  copyDir,
  repoRoot,
  runCommand,
  runCommandAllowFailure,
  sortedJson,
  writeJsonPretty,
} from "./process.ts";

export const RUNTIME_PEER_DEPENDENCIES = {
  typescript: "6.0.3",
  vite: "^8.0.0",
  "vite-plus": "0.1.24",
  vue: "3.5.34",
};

// These complete scripts retain the original Rust command's child-process bytes.
export const NATIVE_RUNTIME_SCRIPT = `
const required = require("@vizejs/native");
(async () => {
  const imported = await import("@vizejs/native");
  const importedNative = imported.default ?? imported;
  for (const [label, native] of [["require", required], ["import", importedNative]]) {
    if (typeof native.compileSfc !== "function") throw new Error(\`compileSfc missing from \${label} smoke\`);
    const result = native.compileSfc(
      '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
      { filename: "Smoke.vue", isTs: true },
    );
    if (!result || result.errors.length > 0 || typeof result.code !== "string" || result.code.length === 0) {
      throw new Error(\`compileSfc \${label} runtime smoke failed\`);
    }
  }
})().catch((error) => { console.error(error); process.exit(1); });
`;
export const INIT_SCRIPT = `
import { pathToFileURL } from "node:url";
import path from "node:path";
const context = JSON.parse(process.argv[1]);
const helper = (name) => import(pathToFileURL(path.join(context.repoRoot, "tools/support/compat/npm", name)));
const { runInitTypecheckChecks } = await helper("smoke-release-init-typecheck.mjs");
const { runFreshProjectInitChecks } = await helper("smoke-release-init-fresh.mjs");
context.packed = new Map(Object.entries(context.packed));
context.compatiblePacked = new Set(context.compatiblePacked);
context.versions = new Map(Object.entries(context.versions));
runInitTypecheckChecks(context.installDir, context.vizeBin, context.repoRoot, context.peers);
runFreshProjectInitChecks(context);
`;

function runInitChecks(
  installDir: string,
  vizeBin: string,
  packages: readonly PackageInfo[],
  allPackages: readonly PackageInfo[],
  tempDir: string,
): void {
  const context = {
    installDir,
    vizeBin,
    repoRoot: repoRoot(),
    tempDir,
    peers: RUNTIME_PEER_DEPENDENCIES,
    packed: Object.fromEntries(allPackages.map((entry) => [entry.name, entry.tarball])),
    compatiblePacked: packages.map((entry) => entry.name),
    versions: Object.fromEntries(allPackages.map((entry) => [entry.name, entry.version])),
  };
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    COREPACK_HOME: path.join(tempDir, "corepack"),
    COREPACK_ENABLE_DOWNLOAD_PROMPT: "0",
  };
  for (const key of ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"])
    delete env[key];
  const output = spawnSync(
    process.env.NODE_BIN ?? "node",
    ["--input-type=module", "-e", INIT_SCRIPT, JSON.stringify(sortedJson(context))],
    { cwd: installDir, env, encoding: "utf8", maxBuffer: Infinity },
  );
  if (output.error) throw new Error(`cannot run installed init oracles: ${output.error.message}`);
  process.stdout.write(output.stdout);
  if (output.status !== 0) {
    const status = output.signal
      ? `signal: ${constants.signals[output.signal]} (${output.signal})`
      : `${process.platform === "win32" ? "exit code" : "exit status"}: ${output.status ?? 1}`;
    throw new Error(`installed init oracles failed with ${status}\n${output.stderr}`);
  }
}

export function runRuntimeChecks(
  installDir: string,
  packages: readonly PackageInfo[],
  allPackages: readonly PackageInfo[],
  tempDir: string,
): void {
  writeRuntimeSmokeProject(installDir);
  const node = process.env.NODE_BIN ?? "node";
  if (packages.some((entry) => entry.name === "@vizejs/native")) {
    runCommand(node, ["-e", NATIVE_RUNTIME_SCRIPT], installDir);
    console.log("runtime: @vizejs/native require/import compileSfc");
  }
  if (packages.some((entry) => entry.name === "vize")) {
    const vizeBin = resolveInstalledBin(installDir, "vize", "vize");
    runCommand(node, [vizeBin, "--version"], installDir);
    console.log("runtime: vize --version");
    runCommand(
      node,
      [vizeBin, "check", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
      installDir,
    );
    console.log("runtime: vize check");
    runCommand(
      node,
      [vizeBin, "lint", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
      installDir,
    );
    console.log("runtime: vize lint");
    runInitChecks(installDir, vizeBin, packages, allPackages, tempDir);
  }
  if (packages.some((entry) => entry.name === "@vizejs/vite-plugin")) {
    const viteBin = resolveInstalledBin(installDir, "vite", "vite");
    runCommand(node, [viteBin, "build"], installDir);
    console.log("runtime: @vizejs/vite-plugin vite build");
  }
}

function writeText(filename: string, source: string): void {
  try {
    fs.mkdirSync(path.dirname(filename), { recursive: true });
  } catch (error) {
    throw new Error(`cannot create ${path.dirname(filename)}: ${String(error)}`);
  }
  try {
    fs.writeFileSync(filename, source);
  } catch (error) {
    throw new Error(`cannot write ${filename}: ${String(error)}`);
  }
}

export function writeRuntimeSmokeProject(installDir: string): void {
  const sourceDir = path.join(installDir, "src");
  try {
    fs.mkdirSync(sourceDir, { recursive: true });
  } catch (error) {
    throw new Error(`cannot create ${sourceDir}: ${String(error)}`);
  }
  writeText(
    path.join(installDir, "index.html"),
    '<div id="app"></div><script type="module" src="/src/main.ts"></script>\n',
  );
  writeJsonPretty(path.join(installDir, "tsconfig.json"), {
    compilerOptions: {
      lib: ["ES2022", "DOM", "DOM.Iterable"],
      module: "ESNext",
      moduleResolution: "Bundler",
      strict: true,
      target: "ES2022",
      types: [],
    },
    include: ["src/**/*.ts", "src/**/*.vue"],
  });
  writeText(
    path.join(installDir, "vite.config.mjs"),
    'import { defineConfig } from "vite";\nimport vize from "@vizejs/vite-plugin";\n\nexport default defineConfig({\n  plugins: [vize()],\n  build: { outDir: "dist", emptyOutDir: true },\n});\n',
  );
  writeText(
    path.join(sourceDir, "App.vue"),
    '<template>\n  <button class="smoke" @click="count++">{{ label }} {{ count }}</button>\n</template>\n\n<script setup lang="ts">\nimport { ref } from "vue";\n\nconst label: string = "vize smoke";\nconst count = ref(0);\n</script>\n\n<style scoped>\n.smoke {\n  color: #0f766e;\n}\n</style>\n',
  );
  writeText(
    path.join(sourceDir, "main.ts"),
    'import { createApp } from "vue";\nimport App from "./App.vue";\n\ncreateApp(App).mount("#app");\n',
  );
}

export function assertInstalledMapperContract(installDir: string): void {
  const packageRoot = path.join(installDir, "node_modules/vize");
  let stat: fs.Stats;
  try {
    stat = fs.lstatSync(packageRoot);
  } catch (error) {
    throw new Error(`cannot stat ${packageRoot}: ${String(error)}`);
  }
  if (stat.isSymbolicLink()) throw new Error(`${packageRoot} must not be a symlink`);
  const manifest = readPackageJson(packageRoot);
  const typescript = manifest.typescript;
  const contract =
    typescript !== null && typeof typescript === "object" && !Array.isArray(typescript)
      ? (typescript as Record<string, unknown>).contentMapper
      : undefined;
  if (
    !isDeepStrictEqual(contract, {
      exec: ["node", "./bin/vize", "content-mapper"],
      compilerOptions: ["noUnusedLocals"],
    })
  ) {
    throw new Error(
      `${path.join(packageRoot, "package.json")} must expose the production typescript.contentMapper contract`,
    );
  }
  const mapperPath = path.join(packageRoot, "bin/vize");
  if (!fs.statSync(mapperPath, { throwIfNoEntry: false })?.isFile())
    throw new Error(`${mapperPath} must exist`);
  if (process.platform !== "win32")
    assertExecutable(mapperPath, `${mapperPath} must be executable`);
}

export function runInstalledContentMapperChecks(installDir: string): void {
  const tsgo = process.env.VIZE_TEST_CONTENT_MAPPER_TSGO ?? "";
  if (tsgo === "") return;
  assertInstalledMapperContract(installDir);
  const projectDir = path.join(installDir, "content mapper project with spaces");
  copyDir(path.join(repoRoot(), "crates/vize/tests/fixtures/content_mapper_project"), projectDir);
  const common = ["--runExternalCode", "--pretty", "false"];
  runCommand(tsgo, [...common, "--noEmit", "-p", "tsconfig.json"], projectDir);
  const broken = runCommandAllowFailure(
    tsgo,
    [...common, "--noEmit", "-p", "tsconfig.error.json"],
    projectDir,
  );
  if (broken.status === 0) throw new Error("installed mapper error fixture unexpectedly passed");
  const brokenOutput = `${broken.stdout}\n${broken.stderr}`;
  for (const [file, code] of [
    ["errors/Broken.vue", "TS2322"],
    ["errors/JavaScriptConsumer.js", "TS2322"],
    ["errors/JsxConsumer.jsx", "TS2322"],
    ["src/Unused.vue", "TS6133"],
  ]) {
    if (!brokenOutput.split(/\r?\n/u).some((line) => line.includes(file) && line.includes(code)))
      throw new Error(`${file} did not report ${code}:\n${brokenOutput}`);
  }
  runCommand(tsgo, [...common, "-p", "tsconfig.emit.json"], projectDir);
  if (
    !["App.d.vue.ts", "App.vue.d.ts"].some((name) =>
      fs.existsSync(path.join(projectDir, "dist", name)),
    )
  )
    throw new Error("installed mapper did not emit an App.vue declaration");
  if (!fs.existsSync(path.join(projectDir, "dist/main.d.ts")))
    throw new Error("installed mapper did not emit dist/main.d.ts");
  try {
    fs.copyFileSync(
      path.join(projectDir, "consumer/verify.ts"),
      path.join(projectDir, "dist/verify.ts"),
    );
  } catch (error) {
    throw new Error(`cannot stage declaration consumer: ${String(error)}`);
  }
  const consumerArgs = [
    "--ignoreConfig",
    "--noEmit",
    "--strict",
    "--module",
    "preserve",
    "--moduleResolution",
    "bundler",
    "--allowArbitraryExtensions",
    "--pretty",
    "false",
    "dist/verify.ts",
  ];
  runCommand(tsgo, consumerArgs, projectDir);
  const tsc = path.join(installDir, "node_modules/typescript/bin/tsc");
  runCommand(process.env.NODE_BIN ?? "node", [tsc, ...consumerArgs], projectDir);
  console.log("runtime: packed vize Content Mapper check and declaration emit");
}
