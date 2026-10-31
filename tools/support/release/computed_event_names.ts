import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  assertCustody,
  assertDeclarationCustody,
  captureDeclarationGraph,
  fileImage,
  inside,
  sha256,
} from "./computed_event_custody.ts";
export { assertCustody, fileImage, sha256, type FileImage } from "./computed_event_custody.ts";

export const checkerVersion = "0.20.1-ts7.1.0-beta";
export const checkerVersionOutput =
  "Version 7.1.0-beta (tsrs 0.20.1-ts7.1.0-beta, microsoft/TypeScript@3e9236361fa9)\n";
export const vueVersion = "3.5.44";
export const sourcePins = {
  "tsconfig.json": "e8a76433f9507dde49cc98f92d9d07f477d778347a810ffbc2462bec177a40cb",
  "src/Child.vue": "955ed75ef099fadfb5101550ac70c1d5ad0fab5e8de16933e5cebeb82db94b7c",
  "src/Probe.vue": "37f852e8cb0d45d4484404e91f45f024454be0326eccc823af81e957cb6a2632",
} as const;
export const negativeProbeSha256 =
  "3d564647a4738d7b298249769102d2e2ba51d05e709ee6d8f1bc8ab5567eaac3";
export const negativeOutput =
  "src/Probe.vue(11,34): error TS2322: Type 'number' is not assignable to type 'boolean'.\n";
export const beforeOutputSha256 =
  "8339daf18f35398693452fc4faf2950dffde11eafa14fb12053a642efb08d1df";
const repository = fileURLToPath(new URL("../../../", import.meta.url));
const fixture = path.join(
  repository,
  "tests/_fixtures/differential/typechecker/dynamic-component-event-names-11140",
);

export const normalizeOutput = (value: string) => value.replace(/\r\n/g, "\n").replace(/\\/g, "/");

export function assertSourcePins(sources: Record<string, Buffer>) {
  assert.deepEqual(Object.keys(sources).sort(), Object.keys(sourcePins).sort());
  for (const [name, expected] of Object.entries(sourcePins)) {
    assert.equal(sha256(sources[name]), expected, `frozen reporter input: ${name}`);
  }
}

type PackageManifest = {
  name: string;
  version: string;
  optionalDependencies?: Record<string, string>;
};
type LockedPackage = { version: string; resolved: string; integrity: string; link?: boolean };
export function assertPublicPackage(
  manifest: PackageManifest,
  locked: LockedPackage,
  name: string,
  version: string,
) {
  assert.equal(manifest.name, name);
  assert.equal(manifest.version, version);
  assert(locked, `missing public lock entry: ${name}`);
  assert.equal(locked.version, version);
  assert.equal(locked.link, undefined, "linked packages are not public installations");
  assert.equal(new URL(locked.resolved).origin, "https://registry.npmjs.org");
  assert.match(locked.integrity, /^sha512-[A-Za-z0-9+/]+={0,2}$/u);
}

export type Observation = {
  id: string;
  status: number | null;
  signal: string | null;
  error: { message: string; code?: string } | null;
  stdout: string;
  stderr: string;
};
export function assertCheckerVersion(row: Observation) {
  assert.equal(row.error, null);
  assert.equal(row.signal, null);
  assert.equal(row.status, 0);
  assert.equal(row.stderr, "");
  assert.equal(normalizeOutput(row.stdout), checkerVersionOutput);
}
export function assertNativeIdentity(token: unknown, imageToken: unknown, identity: unknown) {
  assert.equal(typeof token, "string");
  assert.match(token as string, /^[0-9a-f]{32}$/u);
  assert.equal(imageToken, token, "umbrella and actual loaded image must agree");
  assert.deepEqual(identity, { kind: "native", token });
}
export function assertObservations(phase: "before" | "after", observations: Observation[]) {
  assert.deepEqual(
    observations.map((row) => row.id),
    phase === "before" ? ["baseline"] : ["valid", "invalid", "restored"],
    "every required observation must appear exactly once and in order",
  );
  for (const row of observations) {
    assert.equal(row.error, null, `${row.id}: process failed to run`);
    assert.equal(row.signal, null, `${row.id}: process was signalled`);
    assert.equal(row.stderr, "", `${row.id}: unexpected stderr`);
    assert.equal(row.status, row.id === "invalid" || phase === "before" ? 2 : 0);
    const stdout = normalizeOutput(row.stdout);
    if (phase === "before") {
      assert.equal(sha256(stdout), beforeOutputSha256, "complete historical output changed");
      assert.equal(stdout.match(/: error TS[0-9]+:/gu)?.length, 189);
    } else {
      assert.equal(stdout, row.id === "invalid" ? negativeOutput : "");
    }
  }
}

/** Preserve raw bytes and the complete process outcome before evaluating any oracle. */
export function captureProcess(
  output: string,
  id: string,
  command: string,
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv = process.env,
): Observation {
  assert.match(id, /^[a-z][a-z0-9-]*$/u);
  const started = Date.now();
  const result = spawnSync(command, args, {
    cwd,
    env,
    timeout: 600_000,
    maxBuffer: 32 * 1024 * 1024,
    windowsHide: true,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  writeFileSync(path.join(output, `${id}.stdout.txt`), stdout, { flag: "wx" });
  writeFileSync(path.join(output, `${id}.stderr.txt`), stderr, { flag: "wx" });
  const observation: Observation = {
    id,
    status: result.status,
    signal: result.signal,
    error: result.error
      ? { message: result.error.message, code: (result.error as NodeJS.ErrnoException).code }
      : null,
    stdout: stdout.toString("utf8"),
    stderr: stderr.toString("utf8"),
  };
  writeFileSync(
    path.join(output, `${id}.process.json`),
    JSON.stringify(
      {
        ...observation,
        command,
        args,
        cwd,
        elapsedMs: Date.now() - started,
        stdoutSha256: sha256(stdout),
        stderrSha256: sha256(stderr),
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  return observation;
}

export function parseArguments(args: string[]): {
  version: string;
  output: string;
  phase: "before" | "after";
} {
  const values: Record<string, string> = {};
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index];
    assert(["--version", "--out", "--phase"].includes(key), `unknown argument: ${key}`);
    assert.equal(values[key], undefined, `duplicate argument: ${key}`);
    assert(args[index + 1] && !args[index + 1].startsWith("--"), `missing value: ${key}`);
    values[key] = args[index + 1];
  }
  assert.match(values["--version"] ?? "", /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/u);
  assert(values["--out"] && path.isAbsolute(values["--out"]), "--out must be absolute");
  const phase = values["--phase"] ?? "after";
  assert(phase === "before" || phase === "after");
  if (phase === "before") assert.equal(values["--version"], "0.608.0");
  return { version: values["--version"], output: values["--out"], phase };
}

/** Run npm's JavaScript entry through Node; npm.cmd is never spawned as an executable. */
export function findNpmCli(environment = process.env): string {
  const candidates = [
    environment.npm_execpath,
    path.join(path.dirname(process.execPath), "node_modules/npm/bin/npm-cli.js"),
    path.resolve(path.dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js"),
  ];
  for (const directory of (environment.PATH ?? environment.Path ?? "").split(path.delimiter)) {
    candidates.push(path.join(directory, "node_modules/npm/bin/npm-cli.js"));
    const executable = path.join(directory, process.platform === "win32" ? "npm.cmd" : "npm");
    if (existsSync(executable)) candidates.push(realpathSync(executable));
  }
  for (const candidate of candidates) {
    if (candidate && path.basename(candidate) === "npm-cli.js" && existsSync(candidate)) {
      return realpathSync(candidate);
    }
  }
  throw new Error("Cannot resolve npm-cli.js from the selected Node installation or PATH");
}

export async function main(args = process.argv.slice(2)) {
  const { version, output, phase } = parseArguments(args);
  mkdirSync(path.dirname(output), { recursive: true });
  assert(
    !inside(realpathSync(repository), realpathSync(path.dirname(output))),
    "install outside the source checkout",
  );
  mkdirSync(output); // Exclusive ownership: never reuse or overwrite a previous receipt directory.
  const receiptPath = path.join(output, "receipt.json");
  const harness = fileImage(fileURLToPath(import.meta.url));
  const receipt: Record<string, unknown> = {
    schema: "vize-published-computed-event-names-v1",
    issue: 11140,
    version,
    phase,
    purpose: phase === "before" ? "historical observation" : "public acceptance",
    status: "RUNNING",
    accepted: false,
    node: process.version,
    platform: process.platform,
    arch: process.arch,
    harness,
  };
  const save = () => writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
  save();
  try {
    for (const key of [
      "NODE_OPTIONS",
      "NODE_PATH",
      "VIZE_NATIVE_BINDING_PATH",
      "VIZE_PREFER_WORKSPACE_BINDING",
      "NAPI_RS_NATIVE_LIBRARY_PATH",
      "NAPI_RS_FORCE_WASI",
      "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
      "TSRS_BINARY",
      "TSRS_PATH",
      "TSRS_EXECUTABLE",
      "CORSA_PATH",
      "CORSA_EXECUTABLE",
    ])
      assert(!process.env[key], `public installation forbids runtime override: ${key}`);
    const consumerPath = path.join(output, "consumer");
    mkdirSync(consumerPath);
    const consumer = realpathSync(consumerPath);
    mkdirSync(path.join(consumer, "src"));
    const sources = Object.fromEntries(
      Object.keys(sourcePins).map((name) => [
        name,
        readFileSync(path.join(fixture, `${path.basename(name)}.txt`)),
      ]),
    );
    assertSourcePins(sources);
    receipt.sourcePins = sourcePins;
    for (const [name, bytes] of Object.entries(sources)) {
      writeFileSync(path.join(consumer, name), bytes, { flag: "wx" });
    }
    const packagePath = path.join(consumer, "package.json");
    writeFileSync(
      packagePath,
      JSON.stringify(
        {
          name: "published-computed-events-11140",
          private: true,
          type: "module",
          dependencies: { vize: version, tsrs: checkerVersion, vue: vueVersion },
        },
        null,
        2,
      ) + "\n",
      { flag: "wx" },
    );
    const userConfig = path.join(output, "npm-user.config");
    const globalConfig = path.join(output, "npm-global.config");
    writeFileSync(userConfig, "", { flag: "wx" });
    writeFileSync(globalConfig, "", { flag: "wx" });
    const env = Object.fromEntries(
      Object.entries(process.env).filter(
        ([key]) => !/^npm_config_/iu.test(key) && key.toLowerCase() !== "path",
      ),
    );
    env.PATH = [path.dirname(process.execPath), process.env.PATH ?? process.env.Path ?? ""].join(
      path.delimiter,
    );
    const npm = findNpmCli();
    const npmImage = fileImage(npm);
    receipt.npmCli = npmImage;
    const install = captureProcess(
      output,
      "install",
      process.execPath,
      [
        npm,
        "install",
        "--ignore-scripts",
        "--include=optional",
        "--no-audit",
        "--no-fund",
        "--package-lock=true",
        "--install-strategy=hoisted",
        "--registry=https://registry.npmjs.org",
        `--userconfig=${userConfig}`,
        `--globalconfig=${globalConfig}`,
      ],
      consumer,
      env,
    );
    receipt.install = install;
    save();
    assert.equal(install.error, null);
    assert.equal(install.signal, null);
    assert.equal(install.status, 0, "fresh public npm installation must succeed");
    const lockPath = path.join(consumer, "package-lock.json");
    const lockBytes = readFileSync(lockPath);
    writeFileSync(path.join(output, "package-lock.json"), lockBytes, { flag: "wx" });
    const lock = JSON.parse(lockBytes.toString("utf8"));
    assert.equal(lock.lockfileVersion, 3);
    const require = createRequire(packagePath);
    const images = [
      harness,
      fileImage(fileURLToPath(new URL("./computed_event_custody.ts", import.meta.url))),
      npmImage,
      fileImage(packagePath),
      fileImage(lockPath),
    ];
    const packages: Record<string, unknown>[] = [];
    receipt.packages = packages;
    const identify = (name: string, expected: string) => {
      const manifestPath = path.join(consumer, "node_modules", name, "package.json");
      const manifest: PackageManifest = JSON.parse(readFileSync(manifestPath, "utf8"));
      const locked: LockedPackage = lock.packages[`node_modules/${name}`];
      assertPublicPackage(manifest, locked, name, expected);
      const directory = realpathSync(path.dirname(manifestPath));
      assert(inside(realpathSync(path.join(consumer, "node_modules")), directory));
      images.push(fileImage(manifestPath));
      packages.push({ ...locked, name, version: expected, directory });
      return { directory, manifest };
    };
    const vize = identify("vize", version);
    const native = identify("@vizejs/native", version);
    const checker = identify("tsrs", checkerVersion);
    identify("vue", vueVersion);
    const nativeExports = require("@vizejs/native");
    const loaded = Object.keys(require.cache).filter(
      (name) => name.endsWith(".node") && inside(consumer, name),
    );
    assert.equal(loaded.length, 1, "exactly one installed Vize native provider must load");
    const nativeImage = fileImage(loaded[0]);
    const nativeManifest: PackageManifest = JSON.parse(
      readFileSync(path.join(path.dirname(nativeImage.path), "package.json"), "utf8"),
    );
    assert.match(nativeManifest.name, /^@vizejs\/native-/u);
    assert.equal(native.manifest.optionalDependencies?.[nativeManifest.name], version);
    const provider = identify(nativeManifest.name, version);
    assert(inside(provider.directory, nativeImage.path));
    assert.equal(require.cache[nativeImage.path]?.loaded, true);
    const imageExports = require.cache[nativeImage.path]!.exports;
    const identityHelper = path.join(native.directory, "native-binding-identity.js");
    const bindingIdentity = require(identityHelper).getBindingIdentity(imageExports);
    const nativeBuildToken = nativeExports.getNativeBuildToken();
    assertNativeIdentity(nativeBuildToken, imageExports.getNativeBuildToken(), bindingIdentity);
    receipt.native = {
      ...nativeImage,
      package: nativeManifest.name,
      version,
      nativeBuildToken,
      bindingIdentity,
    };
    const resolver = path.join(checker.directory, "lib/getExePath.js");
    const { default: getExePath } = await import(pathToFileURL(resolver).href);
    const checkerImage = fileImage(getExePath());
    const checkerManifest: PackageManifest = JSON.parse(
      readFileSync(path.join(path.dirname(checkerImage.path), "package.json"), "utf8"),
    );
    assert.match(checkerManifest.name, /^@tsrs\//u);
    assert.equal(checker.manifest.optionalDependencies?.[checkerManifest.name], checkerVersion);
    const checkerProvider = identify(checkerManifest.name, checkerVersion);
    assert(inside(checkerProvider.directory, checkerImage.path));
    assert.equal(
      path.basename(checkerImage.path),
      process.platform === "win32" ? "tsrs.exe" : "tsrs",
    );
    images.push(
      nativeImage,
      checkerImage,
      fileImage(resolver),
      fileImage(identityHelper),
      fileImage(path.join(native.directory, "index.js")),
      fileImage(path.join(native.directory, "native-binding.js")),
      fileImage(path.join(native.directory, "native-targets.js")),
      fileImage(path.join(vize.directory, "bin/vize")),
      fileImage(path.join(vize.directory, "dist/cli.mjs")),
    );
    const declarationGraph = captureDeclarationGraph(vize.directory);
    const declarations = declarationGraph.files;
    images.push(...declarations);
    receipt.declarations = declarations;
    receipt.declarationEdges = declarationGraph.edges;
    const checkCustody = () => {
      assertCustody(images);
      assertDeclarationCustody(vize.directory, declarations, declarationGraph.edges);
    };
    receipt.checker = { ...checkerImage, package: checkerManifest.name, version: checkerVersion };
    receipt.custody = images;
    save();
    const checkerVersionObservation = captureProcess(
      output,
      "checker-version",
      checkerImage.path,
      ["--version"],
      consumer,
      env,
    );
    receipt.checkerVersionObservation = checkerVersionObservation;
    save();
    assertCheckerVersion(checkerVersionObservation);
    checkCustody();
    const observations: Observation[] = [];
    const probe = path.join(consumer, "src/Probe.vue");
    const run = (id: string, expectedProbe: string) => {
      checkCustody();
      const checkSources = () => {
        for (const [name, expected] of Object.entries(sourcePins)) {
          assert.equal(
            sha256(readFileSync(path.join(consumer, name))),
            name === "src/Probe.vue" ? expectedProbe : expected,
          );
        }
      };
      checkSources();
      const row = captureProcess(
        output,
        id,
        checkerImage.path,
        ["--runExternalCode", "--noEmit", "-p", "tsconfig.json"],
        consumer,
        env,
      );
      observations.push(row);
      receipt.observations = observations;
      save();
      checkCustody();
      checkSources();
    };
    if (phase === "before") {
      run("baseline", sourcePins["src/Probe.vue"]);
    } else {
      run("valid", sourcePins["src/Probe.vue"]);
      const negative = sources["src/Probe.vue"]
        .toString("utf8")
        .replace("pressed = true", "pressed = 1");
      assert.equal(sha256(negative), negativeProbeSha256, "independently pinned error control");
      try {
        writeFileSync(probe, negative);
        run("invalid", negativeProbeSha256);
      } finally {
        writeFileSync(probe, sources["src/Probe.vue"]);
      }
      run("restored", sourcePins["src/Probe.vue"]);
    }
    assertObservations(phase, observations);
    receipt.status = "PASSED";
    receipt.accepted = phase === "after";
    save();
    return receipt;
  } catch (error) {
    receipt.status = "FAILED";
    receipt.error =
      error instanceof Error
        ? { name: error.name, message: error.message, stack: error.stack }
        : String(error);
    save();
    throw error;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main()
    .then((receipt) =>
      console.log(JSON.stringify({ status: receipt.status, accepted: receipt.accepted })),
    )
    .catch((error) => {
      console.error(error);
      process.exitCode = 1;
    });
}
