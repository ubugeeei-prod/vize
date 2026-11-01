import assert from "node:assert/strict";
import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { fileImage } from "../computed_event_custody.ts";
import type { Image, InstallationFacts, PackageBase, ReleaseAuthority } from "./model.ts";
import type { Sink } from "./evidence.ts";
import { discard } from "./evidence.ts";
import { array, bytes, object, sha256, text } from "./value.ts";
import { nativeLoaderProgram } from "./native-loader-program.ts";
import { resolveAuthenticatedModule } from "./module-resolution.ts";
import { prohibited } from "./processes.ts";

export interface NativeHostInput {
  platform: "darwin" | "linux";
  arch: "arm64" | "x64";
  musl: boolean | null;
  classifier: "NOT_LINUX" | "FILESYSTEM" | "REPORT" | "UNKNOWN";
  filesystemSha256: string | null;
}
interface NativeBindingInput {
  version: string;
  candidate: string;
  run: string;
  targetPlatform: string;
  packageName: string;
  image: { bytes: number; sha256: string; token: string };
  publicPackages: Array<{ name: string; integrity: string; resolved: string }>;
}
export interface NativeLoadPrerequisite {
  schema: "vize.humanReports11252_11253.nativePrerequisite.v1";
  root: string;
  input: NativeBindingInput;
  host: NativeHostInput;
  targets: string[];
  probeBase: string;
  checkedBases: string[];
  modules: Image[];
  selectedNative: Image;
  selectedOwner: string;
  nonExecutingPrerequisiteOnly: true;
  accepted: false;
  productQualified: false;
  afterQualified: false;
  issueQualified: false;
}
export function readNativeHostInput(): NativeHostInput {
  assert(process.platform === "darwin" || process.platform === "linux");
  assert(process.arch === "arm64" || process.arch === "x64");
  const base = { platform: process.platform, arch: process.arch, filesystemSha256: null };
  if (process.platform === "darwin") return { ...base, musl: false, classifier: "NOT_LINUX" };
  try {
    const raw = readFileSync("/usr/bin/ldd");
    return {
      ...base,
      musl: raw.toString("utf8").includes("musl"),
      classifier: "FILESYSTEM",
      filesystemSha256: sha256(raw),
    };
  } catch {
    if (typeof process.report?.getReport !== "function")
      return { ...base, musl: null, classifier: "UNKNOWN" };
    const report = object(process.report.getReport()),
      header = object(report.header);
    const musl = header.glibcVersionRuntime
      ? false
      : Array.isArray(report.sharedObjects)
        ? report.sharedObjects.some(
            (file) =>
              typeof file === "string" &&
              (file.includes("libc.musl-") || file.includes("ld-musl-")),
          )
        : false;
    return { ...base, musl, classifier: "REPORT" };
  }
}
function targetOrder(host: NativeHostInput): string[] {
  assert(host.arch === "arm64" || host.arch === "x64");
  if (host.platform === "darwin") {
    assert.equal(host.classifier, "NOT_LINUX");
    return ["darwin-universal", "darwin-" + host.arch];
  }
  assert.equal(host.platform, "linux");
  assert(
    host.classifier === "FILESYSTEM" || host.classifier === "REPORT",
    "native loader would need an unobserved child-process detector",
  );
  assert(typeof host.musl === "boolean");
  return ["linux-" + host.arch + (host.musl ? "-musl" : "-gnu")];
}
function bindingInput(authority: ReleaseAuthority, host: NativeHostInput): NativeBindingInput {
  const targets = targetOrder(host),
    packageName = "@vizejs/native-" + targets.at(-1)!;
  const publicSri = object(authority.capsule.npmSRI),
    publicPackages = ["@vizejs/native", packageName].map((name) => {
      const row = object(publicSri[name]);
      return { name, integrity: text(row.integrity), resolved: text(row.tarball) };
    });
  const matches = array(authority.capsule.receipts)
    .map(object)
    .filter((row) => {
      const parsed = object(JSON.parse(bytes(text(row.rawBase64)).toString("utf8")));
      return object(parsed.provider).targetPlatform === host.platform + "-" + host.arch;
    });
  assert.equal(matches.length, 1);
  const origin = matches[0]!,
    raw = bytes(text(origin.rawBase64));
  assert.equal(sha256(raw), origin.sha256);
  assert.equal(String(object(origin.origin).runId), authority.run);
  const receipt = object(JSON.parse(raw.toString("utf8"))),
    provider = object(receipt.provider);
  assert.equal(receipt.schema, "vize.platformVsix.v1");
  assert.equal(receipt.source, authority.candidate);
  assert.equal(provider.source, authority.candidate);
  assert.equal(receipt.version, authority.version);
  assert.equal(provider.releaseVersion, authority.version);
  const images = array(provider.files)
    .map(object)
    .filter((row) => row.path === provider.nodeFile);
  assert.equal(images.length, 1);
  const image = images[0]!;
  assert(Number.isSafeInteger(image.bytes) && Number(image.bytes) > 0);
  const token = text(provider.nativeBuildToken);
  assert.match(token, /^[a-f0-9]{32}$/u);
  assert.match(text(image.sha256), /^[a-f0-9]{64}$/u);
  return {
    version: authority.version,
    candidate: authority.candidate,
    run: authority.run,
    targetPlatform: host.platform + "-" + host.arch,
    packageName,
    image: { bytes: Number(image.bytes), sha256: text(image.sha256), token },
    publicPackages,
  };
}
function expectedPackage(
  packages: readonly PackageBase[],
  name: string,
  input: NativeBindingInput,
): PackageBase {
  const rows = packages.filter((p) => p.name === name && p.version === input.version);
  assert.equal(rows.length, 1);
  const row = rows[0]!,
    expected = input.publicPackages.find((p) => p.name === name);
  assert(expected);
  assert.equal(row.integrity, expected.integrity);
  assert.equal(row.resolved, expected.resolved);
  return row;
}
function member(pkg: PackageBase, name: string): Image {
  const rows = pkg.files.filter((file) => file.path === join(pkg.directory, name));
  assert.equal(rows.length, 1);
  return rows[0]!;
}
function lookupBases(cli: PackageBase, sdk: readonly Image[]): string[] {
  assert.equal(sdk.length, 10);
  for (const file of sdk)
    assert(
      cli.files.some(
        (row) => row.path === file.path && row.bytes === file.bytes && row.sha256 === file.sha256,
      ),
    );
  return [
    ...new Set([
      member(cli, "bin/vize").path,
      join(cli.directory, "package.json"),
      ...sdk.map((f) => f.path),
      ...cli.files.filter((f) => /\.(?:[cm]?js|ts)$/u.test(f.path)).map((f) => f.path),
    ]),
  ];
}
function inspect(
  root: string,
  packages: readonly PackageBase[],
  sdk: readonly Image[],
  input: NativeBindingInput,
  host: NativeHostInput,
  env: NodeJS.ProcessEnv,
  sink: Sink,
): NativeLoadPrerequisite {
  for (const key of prohibited)
    assert.equal(env[key] ?? "", "", "provider/loader override refuses native execution");
  const core = expectedPackage(packages, "@vizejs/native", input),
    platform = expectedPackage(packages, input.packageName, input);
  const cli = packages.filter((p) => p.name === "vize" && p.version === input.version);
  assert.equal(cli.length, 1);
  assert.equal(realpathSync(join(root, "node_modules/vize")), cli[0]!.directory);
  const owners = [core, platform];
  const modules = nativeLoaderProgram.modules.map((program) => {
    const image = member(core, program.name),
      actual = fileImage(image.path);
    sink("actual-public-native-loader-before-execution", {
      program,
      publicMember: image,
      actual,
      sourceKnownOnlyUntilActualPublicArchiveMatched: true,
      bodyBytesExcluded: true,
    });
    assert.equal(image.bytes, program.bytes);
    assert.equal(image.sha256, program.sha256);
    assert.deepEqual(actual, image);
    return image;
  });
  const coreManifest = member(core, "package.json");
  assert.deepEqual(fileImage(coreManifest.path), coreManifest);
  const allowedNative = owners
    .flatMap((pkg) => pkg.files)
    .filter(
      (file) =>
        file.path.endsWith(".node") &&
        file.bytes === input.image.bytes &&
        file.sha256 === input.image.sha256,
    );
  assert(
    allowedNative.length > 0,
    "genuine producer image is not an authenticated public payload member",
  );
  const probeBase = member(cli[0]!, "bin/vize").path;
  const bases = lookupBases(cli[0]!, sdk);
  for (const base of bases) {
    const expected = cli[0]!.files.find((f) => f.path === base);
    assert(expected);
    assert.deepEqual(fileImage(base), expected);
    const result = resolveAuthenticatedModule(
      base,
      "@vizejs/native",
      [core],
      [member(core, "index.js")],
      sink,
    );
    assert(result, "the real CLI/SDK base has no authenticated native wrapper");
  }
  const edges = [
    ["index.js", "./native-binding", "native-binding.js"],
    ["native-binding.js", "./native-targets", "native-targets.js"],
    ["native-binding.js", "./native-binding-identity", "native-binding-identity.js"],
    ["native-targets.js", "./native-binding-identity", "native-binding-identity.js"],
    ["native-targets.js", "./package.json", "package.json"],
    ["index.js", "./native-binding-identity.js", "native-binding-identity.js"],
  ];
  for (const [from, specifier, target] of edges) {
    assert(from && specifier && target);
    assert(
      resolveAuthenticatedModule(
        member(core, from).path,
        specifier,
        owners,
        [member(core, target)],
        sink,
      ),
    );
  }
  const targets = targetOrder(host),
    targetBase = member(core, "native-targets.js").path;
  let selectedNative: Image | null = null;
  // Inspect every prioritized/local/package fallback before any native load.
  for (const target of targets) {
    const local = resolveAuthenticatedModule(
      targetBase,
      "./vize-vitrine." + target + ".node",
      owners,
      allowedNative,
      sink,
    );
    if (!selectedNative && local) selectedNative = local;
    const name = "@vizejs/native-" + target;
    const bundled = resolveAuthenticatedModule(targetBase, name, owners, allowedNative, sink);
    if (bundled) {
      const pkg = owners.find((p) => p.files.some((f) => f.path === bundled.path));
      assert(pkg);
      assert(
        resolveAuthenticatedModule(
          targetBase,
          name + "/package.json",
          owners,
          [member(pkg, "package.json")],
          sink,
        ),
      );
      if (!selectedNative) selectedNative = bundled;
    }
  }
  for (const specifier of ["./vize-vitrine.wasi.cjs", "@vizejs/native-wasm32-wasi"])
    resolveAuthenticatedModule(
      member(core, "native-binding.js").path,
      specifier,
      owners,
      allowedNative,
      sink,
    );
  assert(selectedNative, "default native loader has no authenticated producer image");
  const selectedOwner = owners.find((p) => p.files.some((f) => f.path === selectedNative!.path));
  assert(selectedOwner);
  const proof: NativeLoadPrerequisite = {
    schema: "vize.humanReports11252_11253.nativePrerequisite.v1",
    root,
    input,
    host,
    targets,
    probeBase,
    checkedBases: bases,
    modules,
    selectedNative,
    selectedOwner: selectedOwner.name,
    nonExecutingPrerequisiteOnly: true,
    accepted: false,
    productQualified: false,
    afterQualified: false,
    issueQualified: false,
  };
  sink("native-load-prerequisite-before-probe", proof);
  return proof;
}
export function requireNativeLoadPrerequisite(
  root: string,
  packages: readonly PackageBase[],
  sdk: readonly Image[],
  authority: ReleaseAuthority,
  env: NodeJS.ProcessEnv,
  sink: Sink = discard,
  host = readNativeHostInput(),
): NativeLoadPrerequisite {
  return inspect(root, packages, sdk, bindingInput(authority, host), host, env, sink);
}
export function probeAfterNativePrerequisite<T>(
  input: {
    root: string;
    packages: readonly PackageBase[];
    sdk: readonly Image[];
    authority: ReleaseAuthority;
    env: NodeJS.ProcessEnv;
    sink: Sink;
    host?: NativeHostInput;
  },
  probe: (proof: NativeLoadPrerequisite) => T,
): { proof: NativeLoadPrerequisite; result: T } {
  const proof = requireNativeLoadPrerequisite(
    input.root,
    input.packages,
    input.sdk,
    input.authority,
    input.env,
    input.sink,
    input.host,
  );
  return { proof, result: probe(proof) };
}
export function recheckNativeLoadPrerequisite(
  install: Pick<InstallationFacts, "root" | "packages" | "sdk" | "nativeLoad">,
  env: NodeJS.ProcessEnv,
  sink: Sink = discard,
): void {
  const proof = install.nativeLoad;
  const current = inspect(
    install.root,
    install.packages,
    install.sdk,
    proof.input,
    readNativeHostInput(),
    env,
    sink,
  );
  assert.deepEqual(current, proof);
}
export function validateNativeLoadRecord(
  install: Pick<
    InstallationFacts,
    "root" | "platform" | "arch" | "packages" | "sdk" | "nativeLoad" | "native"
  >,
  authority: ReleaseAuthority,
): void {
  const proof = install.nativeLoad;
  assert.equal(proof.schema, "vize.humanReports11252_11253.nativePrerequisite.v1");
  assert.equal(proof.root, install.root);
  assert.equal(proof.host.platform, install.platform);
  assert.equal(proof.host.arch, install.arch);
  assert.deepEqual(proof.input, bindingInput(authority, proof.host));
  assert.deepEqual(proof.targets, targetOrder(proof.host));
  assert.equal(proof.nonExecutingPrerequisiteOnly, true);
  for (const key of ["accepted", "productQualified", "afterQualified", "issueQualified"] as const)
    assert.equal(proof[key], false);
  const core = expectedPackage(install.packages, "@vizejs/native", proof.input);
  expectedPackage(install.packages, proof.input.packageName, proof.input);
  const cli = install.packages.filter((p) => p.name === "vize" && p.version === authority.version);
  assert.equal(cli.length, 1);
  assert.equal(proof.probeBase, member(cli[0]!, "bin/vize").path);
  assert.deepEqual(proof.checkedBases, lookupBases(cli[0]!, install.sdk));
  assert.deepEqual(
    proof.modules,
    nativeLoaderProgram.modules.map((p) => {
      const file = member(core, p.name);
      assert.equal(file.bytes, p.bytes);
      assert.equal(file.sha256, p.sha256);
      return file;
    }),
  );
  const owner = install.packages.find(
    (p) => p.name === proof.selectedOwner && p.version === authority.version,
  );
  assert(owner && ["@vizejs/native", proof.input.packageName].includes(owner.name));
  assert.equal(proof.selectedNative.bytes, proof.input.image.bytes);
  assert.equal(proof.selectedNative.sha256, proof.input.image.sha256);
  assert(
    owner.files.some(
      (f) =>
        f.path === proof.selectedNative.path &&
        f.bytes === proof.input.image.bytes &&
        f.sha256 === proof.input.image.sha256,
    ),
  );
  assert.equal(install.native.packageName, owner.name);
  assert.equal(install.native.token, proof.input.image.token);
  assert.equal(install.native.identityProbeOnly, true);
  assert.deepEqual(
    { path: install.native.path, bytes: install.native.bytes, sha256: install.native.sha256 },
    proof.selectedNative,
  );
}
