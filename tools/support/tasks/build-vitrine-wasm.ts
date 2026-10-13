/**
 * Builds the Vitrine WebAssembly bindings for a local root task.
 *
 * Usage: node tools/support/tasks/build-vitrine-wasm.ts <target> <out-dir>
 *
 * The pinned `wasm-bindgen` CLI is installed into the wasm-pack cache on first
 * use. Cargo receives an explicit environment so the browser WASM build always
 * aborts on panic and enables the `wasm_js` getrandom backend, and macOS hosts
 * get the SDK paths needed by native build scripts.
 */
import {
  type CommandEnvironment,
  collectStdout,
  commandArguments,
  createDirectory,
  isEntrypoint,
  joinPath,
  pathExists,
  printLine,
  resolvePath,
  runCommand,
  runEntrypoint,
  trimWhitespace,
} from "../command-runtime.ts";

export const wasmBindgenVersion = "0.2.121";

const usage = "Usage: node tools/support/tasks/build-vitrine-wasm.ts <target> <out-dir>";
const wasmTarget = "wasm32-unknown-unknown";

export const macosSdkCandidates = [
  "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk",
  "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
] as const;

function commandStdout(command: string, args: readonly string[], cwd: string): string | undefined {
  let output;
  try {
    output = collectStdout(command, args, { cwd });
  } catch {
    return undefined;
  }
  if (output.exitCode !== 0) return undefined;
  const body = trimWhitespace(output.stdout);
  return body === "" ? undefined : body;
}

export function wasmPackCache(root: string, env: CommandEnvironment = process.env): string {
  const configured = env.WASM_PACK_CACHE;
  return configured === undefined ? joinPath(root, ".cache/wasm-pack") : resolvePath(configured);
}

export function wasmBindgenRoot(root: string, env: CommandEnvironment = process.env): string {
  return joinPath(wasmPackCache(root, env), `.wasm-bindgen-cargo-install-${wasmBindgenVersion}`);
}

export function wasmBindgenBinary(root: string, env: CommandEnvironment = process.env): string {
  return joinPath(wasmBindgenRoot(root, env), "bin/wasm-bindgen");
}

function wasmArtifact(root: string): string {
  return joinPath(root, "target/wasm32-unknown-unknown/release/vize_vitrine.wasm");
}

/**
 * Finds the macOS SDK from `SDKROOT`, then `xcrun`, then the standard
 * Command Line Tools and Xcode locations. Other hosts normally find none.
 */
export function resolveMacosSdkRoot(
  root: string,
  env: CommandEnvironment = process.env,
  candidates: readonly string[] = macosSdkCandidates,
): string | undefined {
  const configured = env.SDKROOT;
  if (configured !== undefined && trimWhitespace(configured) !== "") {
    return resolvePath(configured);
  }
  for (const args of [["--sdk", "macosx", "--show-sdk-path"], ["--show-sdk-path"]]) {
    const value = commandStdout("xcrun", args, root);
    if (value !== undefined) return value;
  }
  return candidates.find((candidate) => pathExists(candidate));
}

export function macosLibraryPath(sdkRoot: string, env: CommandEnvironment = process.env): string {
  const sdkLibraryPath = joinPath(sdkRoot, "usr/lib");
  const inherited = env.LIBRARY_PATH;
  return inherited !== undefined && inherited !== ""
    ? `${sdkLibraryPath}:${inherited}`
    : sdkLibraryPath;
}

export function wasmRustflags(env: CommandEnvironment = process.env): string {
  const flag = '--cfg getrandom_backend="wasm_js"';
  const inherited = env.RUSTFLAGS;
  return inherited !== undefined && trimWhitespace(inherited) !== ""
    ? `${inherited} ${flag}`
    : flag;
}

/** Returns the complete Cargo environment for one invocation. */
export function cargoEnvironment(
  root: string,
  args: readonly string[],
  env: CommandEnvironment = process.env,
  candidates: readonly string[] = macosSdkCandidates,
): CommandEnvironment {
  const cargoEnv: CommandEnvironment = { ...env };
  if (args.includes(wasmTarget)) {
    cargoEnv.RUSTFLAGS = wasmRustflags(env);
    cargoEnv.CARGO_PROFILE_RELEASE_PANIC = "abort";
  }
  const sdkRoot = resolveMacosSdkRoot(root, env, candidates);
  if (sdkRoot !== undefined) {
    cargoEnv.SDKROOT = sdkRoot;
    cargoEnv.LIBRARY_PATH = macosLibraryPath(sdkRoot, env);
    cargoEnv.CC = "clang";
    cargoEnv.CXX = "clang++";
  }
  return cargoEnv;
}

function runCargo(root: string, args: readonly string[]): number {
  return runCommand("cargo", args, { cwd: root, env: cargoEnvironment(root, args) });
}

function ensureDirectory(target: string): void {
  if (!pathExists(target)) createDirectory(target, 0o755);
}

function ensureWasmBindgen(root: string): number {
  if (pathExists(wasmBindgenBinary(root))) return 0;
  ensureDirectory(wasmPackCache(root));
  return runCargo(root, [
    "install",
    "--force",
    "wasm-bindgen-cli",
    "--version",
    wasmBindgenVersion,
    "--root",
    wasmBindgenRoot(root),
  ]);
}

export function main(args: readonly string[]): number {
  if (args.length !== 2) {
    printLine(usage);
    return 1;
  }
  const root = resolvePath(process.cwd());
  const target = args[0]!;
  const outDir = joinPath(root, args[1]!);

  ensureDirectory(outDir);

  const bindgenExit = ensureWasmBindgen(root);
  if (bindgenExit !== 0) return bindgenExit;

  const cargoExit = runCargo(root, [
    "build",
    "--manifest-path",
    joinPath(root, "Cargo.toml"),
    "--release",
    "-p",
    "vize_vitrine",
    "--no-default-features",
    "--features",
    "wasm",
    "--target",
    wasmTarget,
  ]);
  if (cargoExit !== 0) return cargoExit;

  return runCommand(
    wasmBindgenBinary(root),
    [wasmArtifact(root), "--out-dir", outDir, "--target", target],
    { cwd: root },
  );
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => main(commandArguments()));
