import type {
  CargoArtifact,
  SourceReceipt,
  GitIdentity,
} from "./template-alias-paired-11822-types.ts";
import { recordedStage } from "./template-alias-paired-11822-process.ts";
import {
  declarationParser,
  declarationClosure,
  typecheckHarness,
} from "./template-alias-paired-11822-declarations.ts";
import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, basename, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import {
  childEnvironment,
  closureSnapshot,
  evidence,
  json,
  pristine,
  write,
} from "./template-alias-paired-11822-custody.ts";
import {
  copyPublishedPackage,
  requiredDependencyDirectory,
  manifestChanges,
  stageRuntimeDependencies,
} from "./template-alias-paired-11822-packages.ts";

export function verifiedSourcePair(
  base: string,
  head: string,
  sources: { base: GitIdentity; head: GitIdentity },
): Record<string, SourceReceipt> {
  const builds: Record<string, SourceReceipt> = Object.fromEntries(
    [
      ["currentmain", base],
      ["candidate", head],
    ].map(([id, root]) => {
      const receipt = json<SourceReceipt>(join(root, "target/template-alias-11822/receipt.json"));
      assert.equal(receipt.source.sha, sources[id === "currentmain" ? "base" : "head"].sha);
      assert.equal(
        receipt.endpoint,
        "normal Node CLI -> generated SDK loader -> platform NAPI image",
      );
      assert.deepEqual(closureSnapshot([dirname(dirname(receipt.cliPackage))]), receipt.capsule);
      for (const entry of receipt.generatedSdk)
        assert.equal(
          evidence(join(receipt.nativePackage, basename(entry.path))).sha256,
          entry.sha256,
        );
      assert.deepEqual(readFileSync(receipt.image.path), readFileSync(receipt.originalImage.path));
      return [id, receipt];
    }),
  );
  assert.deepEqual(builds.currentmain.artifact.features, builds.candidate.artifact.features);
  assert.deepEqual(builds.currentmain.artifact.profile, builds.candidate.artifact.profile);
  assert.equal(builds.currentmain.rustc, builds.candidate.rustc);
  return builds;
}

export async function buildSource(source: string, sha: string): Promise<SourceReceipt> {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  const identity = pristine(source, sha);
  const target = join(source, "target");
  const out = join(target, "template-alias-11822");
  mkdirSync(out, { recursive: false });
  const commands: {
    id: string;
    command: string;
    args: string[];
    cwd: string;
    processReceipt: string;
  }[] = [];
  const token = randomBytes(16).toString("hex");
  const compilerEnvironment = Object.fromEntries(
    [
      "RUSTUP_TOOLCHAIN",
      "RUSTUP_HOME",
      "CARGO_HOME",
      "RUSTC",
      "RUSTFLAGS",
      "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
      "CARGO_BUILD_JOBS",
      "CC",
      "CXX",
      "AR",
      "LDFLAGS",
    ]
      .filter((key) => process.env[key] !== undefined)
      .map((key) => [key, process.env[key]]),
  );
  const environment = childEnvironment({
    ...compilerEnvironment,
    CARGO_INCREMENTAL: "0",
    CARGO_TARGET_DIR: target,
    VIZE_NATIVE_BUILD_TOKEN: token,
  });
  const run = (
    id: string,
    command: string,
    args: string[],
    cwd: string,
    extra: NodeJS.ProcessEnv = {},
  ) => {
    commands.push({ id, command, args, cwd, processReceipt: join(out, id + ".process.json") });
    writeFileSync(join(out, "commands.json"), JSON.stringify(commands, null, 2) + "\n");
    return recordedStage({
      directory: out,
      id,
      command,
      args,
      cwd,
      environment: { ...environment, ...extra },
      sourceSha: sha,
      maxMilliseconds: id === "napi" ? 720_000 : 300_000,
    });
  };
  const native = join(source, "npm/native");
  const generated = ["index.js", "index.d.ts"].map((name) => ({
    path: join(native, name),
    bytes: readFileSync(join(native, name)),
  }));
  try {
    const rustc = run("rustc", "rustc", ["-Vv"], source);
    run(
      "install",
      "vp",
      [
        "install",
        "--frozen-lockfile",
        "--filter",
        "./npm/cli...",
        "--filter",
        "./npm/native...",
        "--filter",
        ".",
      ],
      source,
    );
    run("vp-version", "vp", ["--version"], source);
    const apiPath = join(
      requiredDependencyDirectory(join(source, "npm/cli"), "typescript"),
      "lib/typescript.js",
    );
    const typescript = declarationParser(apiPath);
    typecheckHarness(
      dirname(fileURLToPath(import.meta.url)),
      apiPath,
      requiredDependencyDirectory(join(source, "npm/cli"), "@types/node"),
    );
    run(
      "declaration-controls",
      process.execPath,
      [
        "--test",
        join(
          dirname(fileURLToPath(import.meta.url)),
          "template-alias-paired-11822-declarations.test.ts",
        ),
      ],
      source,
      { TEMPLATE_ALIAS_TYPESCRIPT_API: apiPath },
    );
    run(
      "helpers",
      process.execPath,
      ["npm/cli/scripts/generate-content-mapper-types.ts", "--check"],
      source,
    );
    pristine(source, sha);
    run(
      "napi",
      "vp",
      [
        "exec",
        "napi",
        "build",
        "--platform",
        "--profile",
        "ci-opt",
        "--manifest-path",
        "../../crates/vize_vitrine/Cargo.toml",
        "-p",
        "vize_vitrine",
        "--features",
        "napi,legacy",
        "--output-dir",
        ".",
        "--",
        "--message-format=json-render-diagnostics",
        "--locked",
        "--config",
        'profile.ci-opt.inherits="release"',
        "--config",
        'profile.ci-opt.lto="thin"',
        "--config",
        "profile.ci-opt.codegen-units=16",
      ],
      native,
    );
    run("native-entrypoint", process.execPath, ["scripts/sync-entrypoint.mjs"], native);
    const messages = readFileSync(join(out, "napi.stdout"), "utf8")
      .split("\n")
      .filter((line) => line.startsWith("{"))
      .map((line) => JSON.parse(line) as Partial<CargoArtifact> & { success?: boolean });
    const artifacts = messages.filter(
      (message): message is CargoArtifact =>
        message.reason === "compiler-artifact" &&
        message.target?.name === "vize_vitrine" &&
        message.target.kind.includes("cdylib"),
    );
    const completed = messages.filter((message) => message.reason === "build-finished");
    assert.equal(completed.length, 1);
    assert.equal(completed[0].success, true);
    assert.equal(artifacts.length, 1);
    const features = artifacts[0].features.toSorted();
    assert.ok(
      features.includes("napi") && features.includes("legacy") && !features.includes("wasm"),
    );
    const generatedSdk = generated.map(({ path }) => evidence(path));
    run("cli-pack", "vp", ["pack"], join(source, "npm/cli"));
    const capsule = join(out, "capsule");
    const cliPackage = join(capsule, "node_modules/vize");
    const nativePackage = join(capsule, "node_modules/@vizejs/native");
    const cliManifest = copyPublishedPackage(join(source, "npm/cli"), cliPackage);
    const nativeManifest = copyPublishedPackage(native, nativePackage);
    write(join(out, "cli-manifest-original.json"), cliManifest);
    write(join(out, "native-manifest-original.json"), nativeManifest);
    assert.ok(existsSync(join(cliPackage, "dist/cli.mjs")), "normal packed CLI entrypoint missing");
    assert.ok(
      readFileSync(join(nativePackage, "index.d.ts")).length > 0,
      "generated native SDK declaration missing",
    );
    const { preparePublishManifest } = await import(
      pathToFileURL(join(source, "tools/support/compat/npm/prepare-publish-manifest.ts")).href
    );
    preparePublishManifest(cliPackage);
    preparePublishManifest(nativePackage);
    const manifestDiffs = {
      cli: manifestChanges(cliManifest, json(join(cliPackage, "package.json"))),
      native: manifestChanges(nativeManifest, json(join(nativePackage, "package.json"))),
    };
    const platformPackage = join(capsule, "node_modules/@vizejs/native-linux-x64-gnu");
    mkdirSync(platformPackage, { recursive: true });
    const image = join(platformPackage, "vize-vitrine.linux-x64-gnu.node");
    const emitted = artifacts[0].filenames.filter((path) => path.endsWith("/libvize_vitrine.so"));
    assert.equal(emitted.length, 1);
    const originalImage = emitted[0];
    assert.ok(
      originalImage.startsWith(target + "/"),
      "native artifact escaped its independent target",
    );
    assert.deepEqual(
      readFileSync(originalImage),
      readFileSync(join(native, "vize-vitrine.linux-x64-gnu.node")),
    );
    copyFileSync(originalImage, image);
    assert.ok(
      readFileSync(image).includes(Buffer.from(token)),
      "fresh source build token absent from staged image",
    );
    write(join(platformPackage, "package.json"), {
      name: "@vizejs/native-linux-x64-gnu",
      version: nativeManifest.version,
      main: "vize-vitrine.linux-x64-gnu.node",
      os: ["linux"],
      cpu: ["x64"],
    });
    const overrides = {
      "@vizejs/native": nativePackage,
      "@vizejs/native-linux-x64-gnu": platformPackage,
    };
    stageRuntimeDependencies(join(source, "npm/cli"), cliPackage, capsule, overrides);
    stageRuntimeDependencies(native, nativePackage, capsule, overrides);
    const buildDependencies = Object.fromEntries(
      ["typescript", "oxfmt", "vite-plus"].map((name) => [
        name,
        evidence(join(requiredDependencyDirectory(join(source, "npm/cli"), name), "package.json")),
      ]),
    );
    buildDependencies.napi = evidence(
      join(requiredDependencyDirectory(native, "@napi-rs/cli"), "package.json"),
    );
    const receipt: SourceReceipt = {
      source: identity,
      endpoint: "normal Node CLI -> generated SDK loader -> platform NAPI image",
      token,
      rustc,
      compilerEnvironment,
      artifact: artifacts[0],
      cliPackage,
      nativePackage,
      image: evidence(image),
      originalImage: evidence(originalImage),
      generatedSdk,
      physicalDeclarations: {
        cli: declarationClosure(cliPackage, typescript),
        native: declarationClosure(nativePackage, typescript),
      },
      manifestDiffs,
      buildDependencies,
      sourceLock: evidence(join(source, "pnpm-lock.yaml")),
      helpers: evidence(join(cliPackage, "generated/content-mapper-types.d.ts")),
      sharedDeclarationAuthority:
        "checked-in files at this exact source SHA; generator --check covers its implemented core/slot scope, not all emit-helper semantics; a separate canonical emit/unknown-arity qualification is required",
      capsule: closureSnapshot([capsule]),
    };
    write(join(out, "receipt.json"), receipt);
    return receipt;
  } finally {
    for (const entry of generated) writeFileSync(entry.path, entry.bytes);
    pristine(source, sha);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const { values } = parseArgs({
    options: { source: { type: "string" }, sha: { type: "string" } },
  });
  assert.ok(values.source && values.sha);
  buildSource(resolve(values.source), values.sha).catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
