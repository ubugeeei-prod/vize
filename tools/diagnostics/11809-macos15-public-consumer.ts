import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";

// Temporary diagnostic source is distinct from the public package's release source.
const branch = "refs/heads/diagnostic/11809-macos15-public678";
const version = "0.678.0";
const sha256 = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");
const fileHash = (file: string): string => sha256(readFileSync(file));
function object(value: unknown): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}
function text(value: unknown): string {
  assert.equal(typeof value, "string");
  return value as string;
}
function image(file: string): Record<string, unknown> {
  const physical = realpathSync(file);
  const bytes = readFileSync(physical);
  return {
    path: file,
    realpath: physical,
    sha256: sha256(bytes),
    bytes: bytes.length,
    magic: bytes.subarray(0, 4).toString("hex"),
    cpuTypeLE: bytes.length >= 8 ? bytes.readUInt32LE(4) : null,
  };
}

// The same strict TS module is imported only by the three separate observed routes.
function observeImages(logfile: string): void {
  const log = (kind: string, details: Record<string, unknown>): void => {
    appendFileSync(
      logfile,
      JSON.stringify({ kind, pid: process.pid, ppid: process.ppid, ...details }) + "\n",
    );
  };
  log("node-process", {
    image: image(process.execPath),
    node: process.version,
    arch: process.arch,
    platform: process.platform,
    argv: process.argv,
    execArgv: process.execArgv,
    cwd: process.cwd(),
  });
  const originalDlopen = process.dlopen.bind(process);
  process.dlopen = (...args: Parameters<typeof process.dlopen>): void => {
    log("native-before", { image: image(args[1]) });
    originalDlopen(...args);
    const exports = object(object(args[0]).exports);
    const getter = exports.getNativeBuildToken;
    const opaqueBuildToken: unknown =
      typeof getter === "function" ? Reflect.apply(getter, exports, []) : null;
    log("native-after", { image: image(args[1]), opaqueBuildToken, tokenIsSourceSha: false });
  };
  const originalExecve = process.execve?.bind(process);
  assert.ok(originalExecve, "official Node 22.23.3 exposes execve");
  process.execve = (...args: Parameters<typeof originalExecve>): never => {
    log("execve", { image: image(args[0]), argv: args[1] ?? null });
    return originalExecve(...args);
  };
}

interface Run {
  id: string;
  command: string;
  args: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
}

const reported = {
  "package.json":
    '{\n  "name": "vize-prop-diagnostic-repro",\n  "private": true,\n  "dependencies": {\n    "vue": "3.5.44",\n    "typescript": "7.1.0-beta",\n    "vize": "0.678.0",\n    "@vue/content-mapper": "4.0.0-alpha.3",\n    "tsrs": "0.21.0-ts7.1.0-beta"\n  }\n}\n',
  "PropsChild.vue":
    '<script setup lang="ts">\ndefineProps<{ benchCount: number }>();\n</script>\n<template><div /></template>\n',
  "PropsParent.vue":
    '<script setup lang="ts">\nimport PropsChild from "./PropsChild.vue";\n</script>\n<template><PropsChild :bench-count="\'bench-negative\'" /></template>\n',
  "tsconfig.vize-no-shim.json":
    '{\n  "compilerOptions": {\n    "target": "ESNext",\n    "module": "ESNext",\n    "moduleResolution": "Bundler",\n    "strict": true,\n    "jsx": "preserve",\n    "skipLibCheck": true,\n    "types": [],\n    "noEmit": true,\n    "incremental": false\n  },\n  "contentMappers": [\n    {\n      "package": "vize",\n      "extensions": [\n        ".vue"\n      ]\n    }\n  ],\n  "include": [\n    "PropsChild.vue",\n    "PropsParent.vue"\n  ]\n}\n',
};
const originalHashes: Record<string, string> = {
  "package.json": "769659bf039040a9a5ad375ea14780dcb699a9de8c44a4920a15524db07786a4",
  "PropsChild.vue": "16d7b729d60501cc53a1fbea01a86be97a57971c6a935b4ae2c65789a46bd3c1",
  "PropsParent.vue": "3378e19998e3834e265d0cee20899da34edff77b55522331a8252fec363b1525",
  "tsconfig.vize-no-shim.json": "8b8353611a3f97c7e2bac9ba216501d0cf283463acca93f8b8d651967cd252f3",
};
const wrongPacket =
  "PropsParent.vue(4,24): error TS2322: Type 'string' is not assignable to type 'number'.\n";
const missingPacket =
  "PropsParent.vue(5,52): error TS2339: Property 'missing' does not exist on type '{ present: number; }'.\n";

async function main(): Promise<void> {
  assert.equal(process.env.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(process.env.GITHUB_REF, branch);
  assert.match(process.env.GITHUB_SHA ?? "", /^[a-f0-9]{40}$/);
  assert.equal(process.platform, "darwin");
  assert.equal(process.arch, "arm64");
  assert.equal(process.version, "v22.23.3");
  const evidence = join(text(process.env.RUNNER_TEMP), "11809-evidence");
  const prefix = join(evidence, "consumer");
  assert.equal(existsSync(prefix), false, "initially absent consumer; no reused install");
  mkdirSync(prefix, { recursive: true });
  const writeJson = (file: string, value: unknown): void => {
    writeFileSync(join(evidence, file), JSON.stringify(value, null, 2) + "\n");
  };
  const environment = { ...process.env, PATH: dirname(process.execPath) + ":" + process.env.PATH };
  const overrides = Object.entries(environment).filter(
    ([key, value]) => value && /^(NODE_OPTIONS|NODE_PATH|NAPI_RS_|VIZE_|CORSA_|TSRS_)/.test(key),
  );
  writeJson("runtime-context.json", {
    diagnosticBranchSha: process.env.GITHUB_SHA,
    publicPackageVersion: version,
    publicPackageSourceSha: null,
    sourceBinding: "Registry public artifact only; diagnostic branch SHA is not its release source",
    scope: "macOS major 15 contrast; no exact reporter OS minor or M4 hardware claim",
    runId: process.env.GITHUB_RUN_ID,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT,
    runnerImage: {
      os: process.env.ImageOS,
      version: process.env.ImageVersion,
      arch: process.env.RUNNER_ARCH,
    },
    node: { ...image(process.execPath), version: process.version, arch: process.arch },
    overrides,
  });
  assert.deepEqual(overrides, [], "canonical runs have no loader/host overrides");
  const run = (
    id: string,
    command: string,
    args: string[],
    cwd = prefix,
    env = environment,
    timeout = 30_000,
  ): Run => {
    const result = spawnSync(command, args, {
      cwd,
      env,
      encoding: "utf8",
      timeout,
      maxBuffer: 32 * 1024 * 1024,
    });
    const row: Run = {
      id,
      command,
      args,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
    writeFileSync(join(evidence, id + ".stdout.txt"), row.stdout);
    writeFileSync(join(evidence, id + ".stderr.txt"), row.stderr);
    writeJson(id + ".json", row);
    return row;
  };
  const os = run("sw-vers", "/usr/bin/sw_vers", ["-productVersion"]);
  assert.equal(os.status, 0);
  assert.match(os.stdout, /^15\./, "OS major 15 only");
  run("uname", "/usr/bin/uname", ["-a"]);
  run("cpu", "/usr/sbin/sysctl", ["machdep.cpu.brand_string", "hw.model", "hw.ncpu", "hw.memsize"]);
  for (const [file, bytes] of Object.entries(reported)) {
    assert.equal(sha256(bytes), originalHashes[file as keyof typeof originalHashes], file);
    writeFileSync(join(prefix, file), bytes);
  }
  const vizeConfig = reported["tsconfig.vize-no-shim.json"];
  const baseConfig = object(JSON.parse(vizeConfig));
  delete baseConfig.contentMappers;
  writeFileSync(join(prefix, "tsconfig.base.json"), JSON.stringify(baseConfig, null, 2) + "\n");
  writeFileSync(
    join(prefix, "tsconfig.vue-no-shim.json"),
    vizeConfig.replace('"package": "vize"', '"package": "@vue/content-mapper"'),
  );
  const snapshots = (): Record<string, string> =>
    Object.fromEntries(
      [...Object.keys(reported), "tsconfig.base.json", "tsconfig.vue-no-shim.json"].map((file) => [
        file,
        fileHash(join(prefix, file)),
      ]),
    );
  const before = snapshots();
  writeJson("original-input-hashes.json", before);
  const npm = join(dirname(process.execPath), "../lib/node_modules/npm/bin/npm-cli.js");
  const install = run(
    "install",
    process.execPath,
    [npm, "install", "--ignore-scripts", "--no-audit", "--no-fund"],
    prefix,
    environment,
    120_000,
  );
  assert.equal(install.status, 0, install.stdout + install.stderr);
  assert.deepEqual(snapshots(), before, "install preserves original reporter package and inputs");
  const lockPath = join(prefix, "package-lock.json");
  const lockHash = fileHash(lockPath);
  copyFileSync(lockPath, join(evidence, "package-lock.json"));
  const issuer = createRequire(join(prefix, "package.json"));
  const packageInfo = (name: string, want: string): { manifest: string; root: string } => {
    // Resolve exported package.json: physical CLI subpaths are intentionally not exported.
    const manifest =
      name === "vize"
        ? join(prefix, "node_modules/vize/package.json") // The declared root package does not export its manifest.
        : issuer.resolve(name + "/package.json");
    const value = object(JSON.parse(readFileSync(manifest, "utf8")));
    assert.equal(value.name, name);
    assert.equal(value.version, want);
    return { manifest, root: dirname(manifest) };
  };
  const expected = object(object(JSON.parse(reported["package.json"])).dependencies);
  const packages = Object.entries(expected).map(([name, want]) => ({
    name,
    version: want,
    ...packageInfo(name, text(want)),
  }));
  packages.push({ name: "@vizejs/native", version, ...packageInfo("@vizejs/native", version) });
  const nativePackage = packageInfo("@vizejs/native-darwin-arm64", version);
  const nativeFile = join(nativePackage.root, "vize-vitrine.darwin-arm64.node");
  const nativeImage = image(nativeFile);
  assert.equal(
    nativeImage.sha256,
    "c48c86069f81b7df64027f542573d65531e5f92598faad3fb754f1a3fc2adfea",
  );
  const hosts = {
    tsc: join(packageInfo("@typescript/typescript-darwin-arm64", "7.1.0-beta").root, "lib/tsc"),
    tsrs: join(packageInfo("@tsrs/darwin-arm64", "0.21.0-ts7.1.0-beta").root, "tsrs"),
  };
  assert.equal(
    fileHash(hosts.tsc),
    "29f5eb8daa5c8cf6ef1c0cb79fbcf47c0cfdf26edabdf148bb779d0b2d56cebc",
  );
  assert.equal(
    fileHash(hosts.tsrs),
    "687bf9336394b089b3fed3d05e41084f32e5a5229c2642512c29f556af2312f6",
  );
  const helpers = [
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
    "generated/content-mapper-types.d.ts",
  ];
  const vizeRoot = packageInfo("vize", version).root;
  const helperHashes = (): Record<string, string> =>
    Object.fromEntries(helpers.map((file) => [file, fileHash(join(vizeRoot, file))]));
  const helpersBefore = helperHashes();
  writeJson("physical-packages.json", {
    packages,
    hosts: Object.fromEntries(Object.entries(hosts).map(([key, file]) => [key, image(file)])),
    nativeImage,
    helpers: helpersBefore,
    lockHash,
  });
  for (const [host, file] of Object.entries(hosts))
    assert.equal(run("version-" + host, file, ["--version"]).status, 0);
  const metadataResponse = await fetch(
    "https://registry.npmjs.org/@vizejs%2Fnative-darwin-arm64/0.678.0",
  );
  assert.equal(metadataResponse.status, 200);
  const metadata: unknown = await metadataResponse.json();
  writeJson("native-registry-metadata.json", metadata);
  const dist = object(object(metadata).dist);
  const tarballUrl = new URL(text(dist.tarball));
  assert.equal(tarballUrl.origin, "https://registry.npmjs.org");
  const tarResponse = await fetch(tarballUrl);
  assert.equal(tarResponse.status, 200);
  const tarball = Buffer.from(await tarResponse.arrayBuffer());
  const integrity = "sha512-" + createHash("sha512").update(tarball).digest("base64");
  assert.equal(integrity, dist.integrity);
  const lockPackages = object(object(JSON.parse(readFileSync(lockPath, "utf8"))).packages);
  assert.equal(
    object(lockPackages["node_modules/@vizejs/native-darwin-arm64"]).integrity,
    integrity,
  );
  writeFileSync(join(evidence, "native-public.tgz"), tarball);
  const extracted = join(evidence, "native-registry-payload");
  mkdirSync(extracted);
  assert.equal(
    run("extract-public-native", "/usr/bin/tar", [
      "-xzf",
      join(evidence, "native-public.tgz"),
      "-C",
      extracted,
    ]).status,
    0,
  );
  assert.equal(
    fileHash(join(extracted, "package/vize-vitrine.darwin-arm64.node")),
    nativeImage.sha256,
  );
  writeJson("native-sri-receipt.json", {
    url: tarballUrl.href,
    integrity,
    archiveSha256: sha256(tarball),
    nativeImage,
  });

  const canonical: Run[] = [];
  const observed: Run[] = [];
  const mapper = (id: string, host: string, kind: string, cwd = prefix, env = environment): Run =>
    run(
      id,
      process.execPath,
      [
        realpathSync(join(prefix, "node_modules/.bin", host)),
        "--runExternalCode",
        "--noEmit",
        "--pretty",
        "false",
        "-p",
        `tsconfig.${kind}-no-shim.json`,
      ],
      cwd,
      env,
    );
  // Capture all bounded routes before judging packets; a failure never erases the other contrasts.
  try {
    for (const host of ["tsc", "tsrs"])
      for (const kind of ["vize", "vue"])
        canonical.push(mapper("wrong-" + host + "-" + kind, host, kind));
    for (const control of ["valid", "missing"]) {
      const directory = join(prefix, "controls", control);
      mkdirSync(directory, { recursive: true });
      const validParent = reported["PropsParent.vue"].replace("'bench-negative'", "42");
      const controlParent =
        control === "valid"
          ? validParent
          : validParent
              .replace("</script>", "const known = { present: 1 };\n</script>")
              .replace(" /></template>", " />{{ known.missing }}</template>");
      writeFileSync(join(directory, "PropsChild.vue"), reported["PropsChild.vue"]);
      writeFileSync(join(directory, "PropsParent.vue"), controlParent);
      writeFileSync(join(directory, "tsconfig.vize-no-shim.json"), vizeConfig);
      writeJson(control + "-control-inputs.json", {
        child: reported["PropsChild.vue"],
        parent: controlParent,
        config: vizeConfig,
      });
      for (const host of ["tsc", "tsrs"])
        canonical.push(mapper(control + "-" + host, host, "vize", directory));
    }
    const standaloneArgs = [
      join(prefix, "node_modules/vize/bin/vize"),
      "check",
      "--tsconfig",
      "tsconfig.base.json",
      "--corsa-path",
      hosts.tsc,
      "--checkers",
      "1",
    ];
    canonical.push(run("standalone-wrong", process.execPath, standaloneArgs));
    for (const route of ["tsc", "tsrs", "standalone"]) {
      const env = {
        ...environment,
        PUBLIC_PROP_OBSERVATION_LOG: join(evidence, "observed-" + route + ".ndjson"),
        NODE_OPTIONS: `--experimental-strip-types --import=${pathToFileURL(import.meta.filename).href}`,
      };
      observed.push(
        route === "standalone"
          ? run("observed-standalone", process.execPath, standaloneArgs, prefix, env)
          : mapper("observed-" + route, route, "vize", prefix, env),
      );
    }
    assert.equal(canonical.length, 9);
    assert.equal(observed.length, 3);
    for (const row of canonical.filter((row) => row.id !== "standalone-wrong")) {
      assert.equal(row.error, null, row.id);
      assert.equal(row.signal, null, row.id);
      assert.equal(row.status, row.id.startsWith("valid") ? 0 : 2, row.id);
      assert.equal(
        row.stdout,
        row.id.startsWith("valid")
          ? ""
          : row.id.startsWith("missing")
            ? missingPacket
            : wrongPacket,
        row.id,
      );
      assert.equal(row.stderr, "", row.id);
    }
    const standalone = canonical[8];
    assert.ok(standalone);
    assert.equal(standalone.error, null);
    assert.equal(standalone.signal, null);
    assert.equal(standalone.status, 1);
    const standalonePacket = `  error:4:24 [TS2322] Type 'string' is not assignable to type 'number'. (source: ${reported["PropsParent.vue"].trim().split("\n").at(-1)})`;
    const diagnosticLines = (row: Run): string[] =>
      (row.stdout + row.stderr).split("\n").filter((line) => /^\s+(?:error|warning):/.test(line));
    assert.deepEqual(diagnosticLines(standalone), [standalonePacket]);
    assert.ok(standalone.stdout.includes(join(prefix, "PropsParent.vue") + "\n"));
    for (const row of observed) {
      assert.equal(row.error, null, row.id);
      assert.equal(row.signal, null, row.id);
      assert.equal(row.status, row.id === "observed-standalone" ? 1 : 2, row.id);
      if (row.id === "observed-standalone")
        assert.deepEqual(diagnosticLines(row), [standalonePacket]);
      else assert.equal(row.stdout, wrongPacket, row.id);
    }
    const events = (route: string): Record<string, unknown>[] =>
      readFileSync(join(evidence, "observed-" + route + ".ndjson"), "utf8")
        .trim()
        .split("\n")
        .map((line) => object(JSON.parse(line)));
    for (const host of ["tsc", "tsrs"] as const)
      assert.ok(
        events(host).some(
          (event) =>
            event.kind === "execve" && object(event.image).sha256 === fileHash(hosts[host]),
        ),
      );
    for (const route of ["tsc", "tsrs", "standalone"])
      assert.ok(
        events(route).some(
          (event) =>
            event.kind === "native-after" && object(event.image).sha256 === nativeImage.sha256,
        ),
      );
    assert.deepEqual(snapshots(), before);
    assert.deepEqual(helperHashes(), helpersBefore);
    assert.equal(fileHash(nativeFile), nativeImage.sha256);
    assert.equal(fileHash(lockPath), lockHash);
    writeJson("verdict.json", {
      status: "all bounded contracts passed; reporter silent0 not reproduced in this scope",
      canonicalCalls: canonical.length,
      observedCalls: observed.length,
      exactReporterHardware: false,
      sourceRepairClaim: false,
      publicPackageVersion: version,
      diagnosticBranchSha: process.env.GITHUB_SHA,
    });
  } finally {
    writeJson("all-results.json", { canonical, observed });
    writeJson("post-input-hashes.json", snapshots());
  }
}

if (process.env.PUBLIC_PROP_OBSERVATION_LOG) observeImages(process.env.PUBLIC_PROP_OBSERVATION_LOG);
else await main();
