import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { constants } from "node:os";
import { stripTypeScriptTypes } from "node:module";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import { pathToFileURL } from "node:url";
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
  oxlint: "1.78.0",
  typescript: "6.0.3",
  "typescript-native": "npm:typescript@7.1.0-beta",
  vite: "^8.0.0",
  "vite-plus": "0.1.24",
  vue: "3.5.34",
};

export function bunRuntimePeerDependencies(): Record<string, string> {
  const peers: Record<string, string> = { ...RUNTIME_PEER_DEPENDENCIES };
  // The packed CLI already owns the stable native TypeScript platform runtime.
  // Injecting the separate mapper-host beta alias makes Bun correctly prefer
  // that project provider, whose newer API is not the LSP compatibility target.
  delete peers["typescript-native"];
  return peers;
}

type RuntimeKind = "bun" | "node";
export interface RuntimeObservations {
  build: Record<string, string>;
  check: string;
  format: string;
  lint: string;
  lsp: "clean";
  native: string;
  oxlint?: string;
  vite: Record<string, string>;
}

function consumerProgram(
  installDir: string,
  name: "native-check.cjs" | "native-observation.cjs" | "init.mjs" | "lsp.mjs",
): string {
  // CommonJS and ESM must both resolve the freshly installed consumer package.
  const folder = path.join(installDir, ".vize-runtime-check");
  fs.mkdirSync(folder, { recursive: true });
  const file = path.join(folder, name);
  if (name.endsWith(".mjs")) {
    // The installed consumer may use an older Node that cannot strip TypeScript.
    const source = new URL("./programs/" + name.replace(/\.mjs$/u, ".ts"), import.meta.url);
    fs.writeFileSync(file, stripTypeScriptTypes(fs.readFileSync(source, "utf8")));
  } else {
    fs.copyFileSync(new URL("./programs/" + name, import.meta.url), file);
  }
  return file;
}

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
  for (const key of [
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
    "TSRS_PATH",
    "TSRS_EXECUTABLE",
  ])
    delete env[key];
  const output = spawnSync(
    process.env.NODE_BIN ?? "node",
    [consumerProgram(installDir, "init.mjs"), JSON.stringify(sortedJson(context))],
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

function normalizeRuntimeText(text: string, installDir: string): string {
  return text
    .replaceAll(pathToFileURL(installDir).href, "file:///VIZE_PROJECT")
    .replaceAll(installDir, "/VIZE_PROJECT")
    .replaceAll(installDir.replaceAll("\\", "/"), "/VIZE_PROJECT");
}

export function normalizeOxlintObservation(text: string, installDir: string): string {
  const observation = JSON.parse(normalizeRuntimeText(text, installDir)) as Record<string, unknown>;
  // Wall-clock telemetry is not product behavior.  Keep diagnostics, spans,
  // rule/file counts and thread selection exact while excluding only the
  // per-invocation timer from the Node/Bun parity packet.
  delete observation.start_time;
  return JSON.stringify(sortedJson(observation));
}

function treeObservation(directory: string, installDir: string): Record<string, string> {
  const result: Record<string, string> = {};
  const visit = (current: string) => {
    for (const entry of fs
      .readdirSync(current, { withFileTypes: true })
      .sort((a, b) => a.name.localeCompare(b.name))) {
      const filename = path.join(current, entry.name);
      if (entry.isDirectory()) visit(filename);
      else if (entry.isFile()) {
        const relative = path.relative(directory, filename).replaceAll(path.sep, "/");
        const normalized = normalizeRuntimeText(fs.readFileSync(filename, "utf8"), installDir);
        result[relative] = createHash("sha256").update(normalized).digest("hex");
      }
    }
  };
  visit(directory);
  return result;
}

function runLspRuntimeCheck(runtime: string, vizeBin: string, installDir: string) {
  const app = path.join(installDir, "src/App.vue");
  const context = JSON.stringify([runtime, vizeBin, installDir, app]);
  const command = process.env.NODE_BIN ?? "node";
  runCommand(command, [consumerProgram(installDir, "lsp.mjs"), context], installDir);
}

export function runRuntimeChecks(
  installDir: string,
  packages: readonly PackageInfo[],
  allPackages: readonly PackageInfo[],
  tempDir: string,
  kind: RuntimeKind = "node",
  fullProducts = false,
  parityPass = false,
): RuntimeObservations {
  writeRuntimeSmokeProject(installDir);
  const runtime =
    kind === "bun" ? (process.env.BUN_BIN ?? "bun") : (process.env.NODE_BIN ?? "node");
  const runtimePrefix = kind === "bun" ? "runtime (Bun)" : "runtime";
  const observations: RuntimeObservations = {
    build: {},
    check: "",
    format: "",
    lint: "",
    lsp: "clean",
    native: "",
    vite: {},
  };
  if (packages.some((entry) => entry.name === "@vizejs/native")) {
    runCommand(runtime, [consumerProgram(installDir, "native-check.cjs")], installDir);
    observations.native = normalizeRuntimeText(
      runCommand(runtime, [consumerProgram(installDir, "native-observation.cjs")], installDir),
      installDir,
    );
    console.log(`${runtimePrefix}: @vizejs/native require/import compileSfc`);
  }
  if (packages.some((entry) => entry.name === "vize")) {
    const vizeBin = resolveInstalledBin(installDir, "vize", "vize");
    runCommand(runtime, [vizeBin, "--version"], installDir);
    console.log(`${runtimePrefix}: vize --version`);
    observations.check = normalizeRuntimeText(
      runCommand(
        runtime,
        [vizeBin, "check", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
        installDir,
      ),
      installDir,
    );
    console.log(`${runtimePrefix}: vize check`);
    observations.lint = normalizeRuntimeText(
      runCommand(
        runtime,
        [vizeBin, "lint", "src/App.vue", "--format", "json", "--quiet", "--no-config"],
        installDir,
      ),
      installDir,
    );
    console.log(`${runtimePrefix}: vize lint`);
    if (fullProducts) {
      runCommand(runtime, [vizeBin, "fmt", "src/Format.vue", "--write", "--no-config"], installDir);
      observations.format = fs.readFileSync(path.join(installDir, "src/Format.vue"), "utf8");
      console.log(`${runtimePrefix}: vize fmt`);
      fs.rmSync(path.join(installDir, "vize-build"), { force: true, recursive: true });
      runCommand(
        runtime,
        [vizeBin, "build", "src/App.vue", "--output", "vize-build", "--no-config"],
        installDir,
      );
      observations.build = treeObservation(path.join(installDir, "vize-build"), installDir);
      console.log(`${runtimePrefix}: vize build`);
      runLspRuntimeCheck(runtime, vizeBin, installDir);
      console.log(`${runtimePrefix}: vize lsp`);
    }
    if (kind === "node" && !parityPass)
      runInitChecks(installDir, vizeBin, packages, allPackages, tempDir);
    else {
      runCommand(
        runtime,
        [
          vizeBin,
          "init",
          "--yes",
          "--no-lint",
          "--vite",
          "--fmt",
          "--typecheck",
          "--no-editor",
          "--dry-run",
          "--package-manager",
          "bun",
        ],
        installDir,
      );
      console.log(`${runtimePrefix}: vize init --package-manager bun`);
    }
  }
  if (packages.some((entry) => entry.name === "@vizejs/vite-plugin")) {
    const viteBin = resolveInstalledBin(installDir, "vite", "vite");
    runCommand(runtime, [viteBin, "build"], installDir);
    observations.vite = treeObservation(path.join(installDir, "dist"), installDir);
    if (kind === "node") console.log("runtime: @vizejs/vite-plugin vite build");
    else console.log("runtime (Bun): @vizejs/vite-plugin vite build");
  }
  if (fullProducts && packages.some((entry) => entry.name === "oxlint-plugin-vize")) {
    const oxlintVize = resolveInstalledBin(installDir, "oxlint-plugin-vize", "oxlint-vize");
    observations.oxlint = normalizeOxlintObservation(
      runCommand(
        runtime,
        [oxlintVize, "-c", ".oxlintrc.json", "-f", "json", "src/Oxlint.vue"],
        installDir,
      ),
      installDir,
    );
    console.log(`${runtimePrefix}: oxlint-vize`);
  }
  return observations;
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
  writeJsonPretty(path.join(installDir, ".oxlintrc.json"), {
    plugins: ["vue"],
    jsPlugins: ["oxlint-plugin-vize"],
    rules: { "vize/vue/no-v-html": "warn" },
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
  writeText(
    path.join(sourceDir, "Format.vue"),
    '<template><p>{{message}}</p></template><script setup lang="ts">const message:string="format"</script>\n',
  );
  writeText(
    path.join(sourceDir, "Oxlint.vue"),
    '<template>\n  <div v-html="body" />\n</template>\n\n<script setup lang="ts">\nconst body = "<strong>unsafe</strong>";\n</script>\n',
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
      dynamicConfig: true,
      exec: ["node", "./bin/vize", "content-mapper"],
      compilerOptions: [
        "checkJs",
        "noUnusedLocals",
        "rootDir",
        "module",
        "moduleResolution",
        "resolvePackageJsonExports",
        "customConditions",
        "baseUrl",
        "paths",
        "moduleSuffixes",
        "noDtsResolution",
        "preserveSymlinks",
      ],
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
