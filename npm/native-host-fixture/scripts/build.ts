import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createReadStream, existsSync, readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import { build, type Plugin } from "vite-plus";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const repositoryRoot = path.resolve(packageRoot, "../..");
const require = createRequire(import.meta.url);
const maximumGzipBytes = 64 * 1024;

interface CompilerReceipt {
  readonly source: string;
  readonly nativeBuildToken: string;
  readonly sha256: string;
  readonly nodeFile: string;
}

/** Options for compiling the canonical application into its offline execution bundle. */
export interface NativeApplicationBuildOptions {
  /**
   * Explicit existing compiler image used by a local preliminary compatibility check.
   * Actions always verifies the checkout producer receipt and image token.
   *
   * @default the checkout's native image for the current macOS architecture
   */
  readonly compilerPath?: string;
  /**
   * Destination of the generated classic execution bundle.
   *
   * @default "ios/Resources/application.js"
   */
  readonly output?: string;
}

/** Hashes the actual native image without retaining its large contents in memory. */
async function hashFile(file: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(file)) hash.update(chunk);
  return hash.digest("hex");
}

type NativeCompiler = typeof import("@vizejs/native");
interface CompiledSource {
  readonly file: string;
  readonly sha256: string;
}

async function loadCompiler(options: NativeApplicationBuildOptions) {
  const image = path.resolve(
    options.compilerPath ??
      path.join(repositoryRoot, "npm/native", `vize-vitrine.darwin-${process.arch}.node`),
  );
  if (!existsSync(image))
    throw new Error("Build the checkout native compiler before building the first host.");
  process.env.NAPI_RS_NATIVE_LIBRARY_PATH = image;
  const native = require("@vizejs/native") as NativeCompiler;
  const compilerSha256 = await hashFile(image);
  const token = native.getNativeBuildToken();
  const source = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: repositoryRoot,
    encoding: "utf8",
  }).trim();
  const sourceChanges = execFileSync("git", ["status", "--porcelain", "--untracked-files=no"], {
    cwd: repositoryRoot,
    encoding: "utf8",
  }).trim();
  if (process.env.GITHUB_ACTIONS === "true" && sourceChanges !== "") {
    throw new Error("Native host qualification requires a clean tracked source checkout.");
  }
  const receiptFile = process.env.VIZE_NATIVE_BUILD_RECEIPT;
  let compilerReceipt: CompilerReceipt | undefined;
  if (receiptFile) {
    compilerReceipt = JSON.parse(readFileSync(receiptFile, "utf8")) as CompilerReceipt;
    if (
      compilerReceipt.source !== source ||
      compilerReceipt.nativeBuildToken !== token ||
      compilerReceipt.sha256 !== compilerSha256 ||
      compilerReceipt.nodeFile !== path.basename(image)
    ) {
      throw new Error(
        "The compiler producer receipt does not identify this checkout and loaded image.",
      );
    }
  } else if (process.env.GITHUB_ACTIONS === "true") {
    throw new Error(
      "Actions first-host qualification requires the actual source compiler receipt.",
    );
  }

  return { native, compilerSha256, token, source, sourceChanges, compilerReceipt };
}

function sfcPlugin(native: NativeCompiler, compiledSources: Map<string, CompiledSource>): Plugin {
  return {
    name: "vize-native-host:sfc",
    enforce: "pre",
    load(id) {
      if (!id.endsWith(".vue")) return undefined;
      const authored = readFileSync(id, "utf8");
      const logicalFile = path.relative(packageRoot, id).split(path.sep).join("/");
      const descriptor = native.parseSfc(authored, { filename: id, includeAttrs: true });
      if (
        !descriptor.template ||
        !descriptor.scriptSetup ||
        descriptor.script ||
        descriptor.styles.length !== 1
      ) {
        throw new Error(
          "Native fixture components require canonical template, script setup, and style sections.",
        );
      }
      if (descriptor.styles.some(({ content }) => content.trim() !== "")) {
        throw new Error(
          "The first native host supports typed primitive layout; native CSS is not implemented.",
        );
      }
      const output = native.compileSfc(authored, {
        filename: id,
        sourceMap: false,
        customRenderer: true,
        customElements: ["native-view", "native-text", "native-button"],
        scopeId: `data-v-${createHash("sha256").update(logicalFile).digest("hex").slice(0, 12)}`,
        vapor: false,
        ssr: false,
        inlineTemplate: true,
        isProd: true,
        isTs: false,
      });
      if (output.errors.length || output.warnings.length) {
        throw new Error(
          `Native SFC compilation failed: ${[...output.errors, ...output.warnings].join("\n")}`,
        );
      }
      if (output.css?.trim())
        throw new Error("The first native host cannot install generated CSS.");
      compiledSources.set(id, {
        file: logicalFile,
        sha256: createHash("sha256").update(authored).digest("hex"),
      });
      return output.code;
    },
  };
}

async function bundleApplication(
  native: NativeCompiler,
  compiledSources: Map<string, CompiledSource>,
) {
  const result = await build({
    configFile: false,
    root: packageRoot,
    mode: "production",
    logLevel: "warn",
    define: {
      "process.env.NODE_ENV": JSON.stringify("production"),
      __VUE_OPTIONS_API__: "false",
      __VUE_PROD_DEVTOOLS__: "false",
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: "false",
    },
    plugins: [sfcPlugin(native, compiledSources)],
    build: {
      target: "es2022",
      minify: true,
      sourcemap: false,
      write: false,
      emptyOutDir: false,
      lib: {
        entry: path.join(packageRoot, "src/entry.ts"),
        name: "VizeNativeApplication",
        formats: ["iife"],
        fileName: () => "application.js",
      },
    },
  });
  const builds = Array.isArray(result) ? result : [result];
  const emitted = builds[0];
  if (builds.length !== 1 || !emitted || !("output" in emitted))
    throw new Error("Expected one native application build.");
  const chunks = emitted.output.filter((item) => item.type === "chunk");
  const chunk = chunks[0];
  if (
    emitted.output.length !== 1 ||
    chunks.length !== 1 ||
    !chunk ||
    chunk.imports.length ||
    chunk.dynamicImports.length
  ) {
    throw new Error(
      "The first host requires one offline execution bundle without external chunks.",
    );
  }
  return chunk.code;
}

/**
 * Compiles real SFC sources with Vize and bundles their typed native adapter.
 *
 * Non-empty CSS, missing canonical sections, compiler findings, external runtime
 * chunks, and oversized execution bundles fail before generated output replaces
 * the app resource. The generated file is never an authored component source.
 */
export async function buildNativeApplication(options: NativeApplicationBuildOptions = {}) {
  const { native, compilerSha256, token, source, sourceChanges, compilerReceipt } =
    await loadCompiler(options);
  const compiledSources = new Map<string, CompiledSource>();
  const code = await bundleApplication(native, compiledSources);
  const gzipBytes = gzipSync(code, { level: 9 }).byteLength;
  if (gzipBytes > maximumGzipBytes)
    throw new Error("The native application execution bundle exceeds its gzip budget.");
  const output = path.resolve(packageRoot, options.output ?? "ios/Resources/application.js");
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(output, code);
  const receipt = {
    format: "vize.native-host.application-build",
    formatVersion: 1,
    source,
    trackedSourceClean: sourceChanges === "",
    compilerSourceVerified: compilerReceipt !== undefined,
    compilerSha256,
    compilerBuildToken: token,
    sources: [...compiledSources.values()].sort((left, right) =>
      left.file.localeCompare(right.file),
    ),
    sha256: createHash("sha256").update(code).digest("hex"),
    bytes: Buffer.byteLength(code),
    gzipBytes,
    maximumGzipBytes,
    vue: (require("vue/package.json") as { readonly version: string }).version,
  };
  const receiptPath = path.join(packageRoot, ".build/application-build.json");
  mkdirSync(path.dirname(receiptPath), { recursive: true });
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt));
  return { output, receipt, code };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await buildNativeApplication(
    process.env.VIZE_NATIVE_HOST_COMPILER_PATH
      ? { compilerPath: process.env.VIZE_NATIVE_HOST_COMPILER_PATH }
      : {},
  );
}
