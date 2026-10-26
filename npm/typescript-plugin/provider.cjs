"use strict";
const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");
const compareNames = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const ownVersion = require("./package.json").version;
const canonicalFiles = [
  "package.json",
  "index.js",
  "index.d.ts",
  "native-binding.js",
  "native-binding-identity.js",
  "native-targets.js",
  "language-projection.js",
  "language-projection.d.ts",
];
const targets = {
  "x86_64-apple-darwin": ["darwin-x64", "vize-vitrine.darwin-x64.node"],
  "aarch64-apple-darwin": ["darwin-arm64", "vize-vitrine.darwin-arm64.node"],
  "x86_64-pc-windows-msvc": ["win32-x64", "vize-vitrine.win32-x64-msvc.node"],
  "aarch64-pc-windows-msvc": ["win32-arm64", "vize-vitrine.win32-arm64-msvc.node"],
  "x86_64-unknown-linux-gnu": ["linux-x64", "vize-vitrine.linux-x64-gnu.node"],
  "aarch64-unknown-linux-gnu": ["linux-arm64", "vize-vitrine.linux-arm64-gnu.node"],
  "x86_64-unknown-linux-musl": ["alpine-x64", "vize-vitrine.linux-x64-musl.node"],
  "aarch64-unknown-linux-musl": ["alpine-arm64", "vize-vitrine.linux-arm64-musl.node"],
};
function ownExtensionRoot() {
  // Derive the host solely from this module's physical installed package.
  // A workspace tsconfig cannot select or impersonate the bundled provider.
  const directory = fs.realpathSync(__dirname);
  const root = path.resolve(directory, "../../..");
  if (
    path.relative(root, directory).split(path.sep).join("/") !==
    "node_modules/@vizejs/typescript-vue-plugin"
  )
    return undefined;
  try {
    const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
    return manifest.name === "vize" &&
      manifest.publisher === "ubugeeei" &&
      manifest.main === "./dist/extension.cjs" &&
      manifest.contributes?.typescriptServerPlugins?.some(
        (plugin) => plugin.name === "@vizejs/typescript-vue-plugin",
      )
      ? root
      : undefined;
  } catch {
    return undefined;
  }
}
function checkCapabilities(native, helper) {
  if (
    typeof native.generateVueLanguageCode !== "function" ||
    typeof helper.toUtf16LanguageProjection !== "function" ||
    typeof helper.applyDiagnosticDirectives !== "function" ||
    typeof helper.mapLanguageDiagnosticRange !== "function"
  )
    throw new Error("Native projection capabilities are unavailable");
}
function bundledProvider(root) {
  if (JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8")).version !== ownVersion)
    throw new Error("Extension and adapter release versions differ");
  const directory = path.join(root, "native/typescript-projection");
  // The universal VSIX has no provider, so the plugin leaves tsserver untouched.
  if (!fs.existsSync(path.join(directory, "provider.json")))
    return {
      error:
        "This universal VSIX provides component metadata only; install the matching platform VSIX for the native TypeScript service",
      unavailable: true,
    };
  const directoryStat = fs.lstatSync(directory);
  if (!directoryStat.isDirectory() || directoryStat.isSymbolicLink())
    throw new Error("Bundled provider directory is not physical");
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, "provider.json"), "utf8"));
  const target = Object.hasOwn(targets, receipt.rustTarget)
    ? targets[receipt.rustTarget]
    : undefined;
  if (
    receipt.schema !== "vize.vscodeNativeProvider.v1" ||
    receipt.releaseVersion !== ownVersion ||
    !/^[0-9a-f]{40}$/.test(receipt.source) ||
    !/^[0-9a-f]{32}$/.test(receipt.nativeBuildToken) ||
    !target ||
    target[0] !== receipt.targetPlatform ||
    target[1] !== receipt.nodeFile
  )
    throw new Error("Invalid bundled native provider identity");
  const expected = [...canonicalFiles, receipt.nodeFile, "build.json"].sort(compareNames);
  if (
    JSON.stringify(receipt.files.map((file) => file.path).sort(compareNames)) !==
      JSON.stringify(expected) ||
    JSON.stringify(fs.readdirSync(directory).sort(compareNames)) !==
      JSON.stringify([...expected, "provider.json"].sort(compareNames))
  )
    throw new Error("Unexpected bundled native provider inventory");
  for (const file of receipt.files) {
    const filename = path.join(directory, file.path);
    const stat = fs.lstatSync(filename);
    if (
      !stat.isFile() ||
      stat.isSymbolicLink() ||
      stat.size !== file.bytes ||
      stat.size > (file.path === receipt.nodeFile ? 100_000_000 : 2_000_000) ||
      createHash("sha256").update(fs.readFileSync(filename)).digest("hex") !== file.sha256
    )
      throw new Error(`Bundled native provider changed: ${file.path}`);
  }
  const metadata = require(path.join(directory, "package.json"));
  if (metadata.name !== "@vizejs/native" || metadata.version !== ownVersion)
    throw new Error("Bundled native package version mismatch");
  const build = require(path.join(directory, "build.json"));
  if (
    build.schema !== "vize.nativeBuildReceipt.v1" ||
    build.source !== receipt.source ||
    build.releaseVersion !== ownVersion ||
    build.rustTarget !== receipt.rustTarget ||
    build.nodeFile !== receipt.nodeFile ||
    build.nativeBuildToken !== receipt.nativeBuildToken ||
    build.bytes !== receipt.files.find((file) => file.path === receipt.nodeFile).bytes ||
    build.sha256 !== receipt.files.find((file) => file.path === receipt.nodeFile).sha256
  )
    throw new Error("Bundled native producer receipt mismatch");
  // This is an implementation dependency selected by the signed extension,
  // so load its exact image. Workspace paths and global native loader overrides
  // do not substitute another image into this default service.
  const { requireWithBindingIdentity } = require(
    path.join(directory, "native-binding-identity.js"),
  );
  const entry = path.join(directory, receipt.nodeFile);
  const native = requireWithBindingIdentity(require, entry);
  if (native.getNativeBuildToken?.() !== receipt.nativeBuildToken)
    throw new Error("Loaded native image token differs from the bundled producer");
  const helper = require(path.join(directory, "language-projection.js"));
  checkCapabilities(native, helper);
  return {
    native,
    ...helper,
    path: entry,
    version: ownVersion,
    origin: "bundled",
    source: receipt.source,
    target: receipt.targetPlatform,
    nativeBuildToken: receipt.nativeBuildToken,
  };
}
function loadNative(projectRoot, nativePath) {
  try {
    const extension = ownExtensionRoot();
    if (extension) return bundledProvider(extension);
    let entry;
    try {
      entry =
        nativePath === undefined
          ? require.resolve("@vizejs/native")
          : require.resolve(path.resolve(projectRoot || process.cwd(), nativePath));
    } catch (cause) {
      // An absent default package is not a projection failure. An explicit path still is.
      if (nativePath === undefined && cause?.code === "MODULE_NOT_FOUND")
        return { error: cause.message, unavailable: true };
      throw cause;
    }
    const native = require(entry),
      directory = path.dirname(entry);
    const metadata = require(path.join(directory, "package.json"));
    if (metadata.name !== "@vizejs/native")
      throw new Error("nativePath must identify @vizejs/native");
    if (nativePath === undefined && metadata.version !== ownVersion)
      throw new Error("The default native provider must match the plugin release");
    const helper = require(path.join(directory, "language-projection.js"));
    checkCapabilities(native, helper);
    return {
      native,
      ...helper,
      path: entry,
      version: metadata.version,
      origin: nativePath === undefined ? "package" : "explicit",
    };
  } catch (cause) {
    return { error: cause instanceof Error ? cause.message : String(cause) };
  }
}
module.exports = { loadNative };
