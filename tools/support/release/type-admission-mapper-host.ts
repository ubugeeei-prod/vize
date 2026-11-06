import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { asRecord as record, asString as string } from "../json-value.ts";

export const MAPPER_HOST_VERSION = "7.1.0-beta";
export const MAPPER_HOST_GIT = "3e9236361fa97a9e6c685f4833ac457ae2d8c50a";
const MAPPER_HOST_COMMON = {
  name: "typescript",
  integrity:
    "sha512-cjWjZAJDsmZzsNC5GwHNlW0YPM65jZa5oR0KyQcYX3KSVIfTvtBq//vewISyatKk4OLi5grZjHhIVE9fdrw1UQ==",
  tarball: "https://registry.npmjs.org/typescript/-/typescript-7.1.0-beta.tgz",
} as const;
export const MAPPER_HOST_TARGETS = {
  "linux-x64": {
    platform: "linux",
    arch: "x64",
    package: {
      name: "@typescript/typescript-linux-x64",
      integrity:
        "sha512-kcD51ihf+pnI5KT4uQQYG04OpwznaZYBlIQhDhl8fflFd27NktxXsUeguxZyvUMyWfum7IF2bBS6Jv7ox0AgFQ==",
      tarball:
        "https://registry.npmjs.org/@typescript/typescript-linux-x64/-/typescript-linux-x64-7.1.0-beta.tgz",
    },
    image: {
      bytes: 24522912,
      sha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    },
  },
  "darwin-arm64": {
    platform: "darwin",
    arch: "arm64",
    package: {
      name: "@typescript/typescript-darwin-arm64",
      integrity:
        "sha512-q7mxblcyX0E7hXsY4k+Q6KWKK5mGmfVq+BDesdTgRVgjodk27aIxER77Zyj9+TgSTlCje8F7+JM6f6hl2BuNdw==",
      tarball:
        "https://registry.npmjs.org/@typescript/typescript-darwin-arm64/-/typescript-darwin-arm64-7.1.0-beta.tgz",
    },
    image: {
      bytes: 24244594,
      sha256: "29f5eb8daa5c8cf6ef1c0cb79fbcf47c0cfdf26edabdf148bb779d0b2d56cebc",
    },
  },
} as const;
export const MAPPER_HOST_IMAGE = MAPPER_HOST_TARGETS["linux-x64"].image;
export const MAPPER_HOST_PACKAGES = [
  MAPPER_HOST_COMMON,
  MAPPER_HOST_TARGETS["linux-x64"].package,
] as const;
type MapperHostPackage = {
  readonly name: string;
  readonly integrity: string;
  readonly tarball: string;
};

export function mapperHostTarget(platform = process.platform, arch = process.arch) {
  const target = Object.values(MAPPER_HOST_TARGETS).find(
    (candidate) => candidate.platform === platform && candidate.arch === arch,
  );
  assert.ok(target, `unsupported mapper host target: ${platform}-${arch}`);
  return target;
}

export function validateMapperHostMetadata(
  value: unknown,
  index: number,
  packages: readonly MapperHostPackage[] = MAPPER_HOST_PACKAGES,
): void {
  const expected = packages[index];
  assert.ok(expected);
  const metadata = record(value),
    dist = record(metadata.dist);
  assert.equal(metadata.name, expected.name);
  assert.equal(metadata.version, MAPPER_HOST_VERSION);
  assert.equal(metadata.gitHead, MAPPER_HOST_GIT);
  assert.equal(dist.integrity, expected.integrity);
  assert.equal(dist.tarball, expected.tarball);
  if (index === 0) {
    const binaryPackage = packages[1];
    assert.ok(binaryPackage);
    assert.equal(
      record(metadata.optionalDependencies)[binaryPackage.name],
      MAPPER_HOST_VERSION,
    );
  }
}

function image(file: string): { file: string; bytes: number; sha256: string } {
  const physical = fs.realpathSync(file),
    bytes = fs.readFileSync(physical);
  return {
    file: physical,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
  };
}
function json(file: string): unknown {
  return JSON.parse(fs.readFileSync(file, "utf8")) as unknown;
}

export async function bootstrapMapperHost(workspace: string, output: string): Promise<void> {
  const target = mapperHostTarget();
  const packages = [MAPPER_HOST_COMMON, target.package] as const;
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  for (const name of [
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "NODE_AUTH_TOKEN",
    "NPM_TOKEN",
    "NODE_OPTIONS",
    "NODE_PATH",
  ])
    assert.ok(!process.env[name], name + " must not reach the independent mapper-host installer");
  const temporary = fs.realpathSync(string(process.env.RUNNER_TEMP));
  const root = path.resolve(workspace),
    directory = path.resolve(output);
  assert.equal(
    path.dirname(root),
    temporary,
    "mapper host must be a separate fresh RUNNER_TEMP child",
  );
  assert.equal(fs.existsSync(root), false, "never reuse or replace a mapper-host graph");
  assert.ok(directory.startsWith(temporary + path.sep) && directory !== root);
  assert.ok(
    !directory.startsWith(root + path.sep),
    "proof must not be part of the installed graph",
  );
  assert.equal(fs.existsSync(directory), false, "never overwrite mapper-host proof");
  fs.mkdirSync(directory);
  const write = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n", {
      flag: "wx",
    });
  const receipt: Record<string, unknown> = {
    schema: "vize.typeAdmission.mapperHostSetup.v1",
    role: "hosted-content-mapper",
    sdkVersion: MAPPER_HOST_VERSION,
    workspace: root,
    workflowSha: process.env.GITHUB_SHA,
    runId: process.env.GITHUB_RUN_ID,
    platform: process.platform,
    arch: process.arch,
    targetPackage: target.package.name,
    node: process.version,
    status: "preparing",
    hostedCapability: "not-run",
    cliAndLspProvider: "existing CPP 7.0.2 remains unchanged",
  };
  let serial = 0;
  const invoke = (command: string, args: string[], cwd = root): string => {
    const result = spawnSync(command, args, {
      cwd,
      env: process.env,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 8 * 1024 * 1024,
    });
    const prefix = String(++serial).padStart(2, "0");
    const packet = {
      command,
      args,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error ? { name: result.error.name, message: result.error.message } : null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
    write(prefix + "-process.json.txt", packet);
    assert.equal(packet.error, null, prefix);
    assert.equal(packet.signal, null, prefix);
    assert.equal(packet.status, 0, JSON.stringify(packet));
    return packet.stdout;
  };
  try {
    const metadata: unknown[] = [];
    for (let index = 0; index < packages.length; index++) {
      const expected = packages[index];
      assert.ok(expected);
      const url =
        "https://registry.npmjs.org/" +
        encodeURIComponent(expected.name) +
        "/" +
        MAPPER_HOST_VERSION;
      const response = await fetch(url, { signal: AbortSignal.timeout(30_000) });
      assert.equal(response.status, 200, url);
      const raw = await response.text();
      fs.writeFileSync(path.join(directory, "registry-" + index + ".json.txt"), raw, {
        flag: "wx",
      });
      const value: unknown = JSON.parse(raw);
      validateMapperHostMetadata(value, index, packages);
      metadata.push({ url, metadata: value });
    }
    fs.mkdirSync(root);
    fs.writeFileSync(
      path.join(root, "package.json"),
      JSON.stringify(
        {
          name: "vize-isolated-mapper-host",
          private: true,
          version: "0.0.0",
          dependencies: { typescript: MAPPER_HOST_VERSION },
        },
        null,
        2,
      ) + "\n",
      { flag: "wx" },
    );
    const options = [
      "--ignore-scripts",
      "--include=optional",
      "--no-audit",
      "--no-fund",
      "--registry=https://registry.npmjs.org",
    ];
    invoke("npm", ["install", "--package-lock-only", ...options]);
    const lock = record(json(path.join(root, "package-lock.json"))),
      lockPackages = record(lock.packages);
    const rows: unknown[] = [];
    for (const expected of packages) {
      const row = record(lockPackages["node_modules/" + expected.name]);
      assert.equal(row.version, MAPPER_HOST_VERSION);
      assert.equal(row.integrity, expected.integrity);
      assert.equal(row.resolved, expected.tarball);
      rows.push({ name: expected.name, lock: row });
    }
    invoke("npm", ["ci", ...options]);
    for (const expected of packages)
      assert.equal(
        record(json(path.join(root, "node_modules", expected.name, "package.json"))).version,
        MAPPER_HOST_VERSION,
      );
    const executable = image(path.join(root, "node_modules", target.package.name, "lib/tsc"));
    assert.equal(executable.bytes, target.image.bytes);
    assert.equal(executable.sha256, target.image.sha256);
    const version = invoke(executable.file, ["--version"]);
    assert.match(version.trim(), /^Version 7\.1\.0-beta$/);
    invoke(executable.file, ["--help"]);
    receipt.registry = metadata;
    receipt.lockRows = rows;
    receipt.lock = image(path.join(root, "package-lock.json"));
    receipt.manifests = packages.map((item) =>
      image(path.join(root, "node_modules", item.name, "package.json")),
    );
    receipt.executable = executable;
    receipt.versionOutput = version;
    receipt.status = "prepared";
    fs.copyFileSync(
      path.join(root, "package-lock.json"),
      path.join(directory, "package-lock.json.txt"),
      fs.constants.COPYFILE_EXCL,
    );
    write("manifest.json.txt", receipt);
  } catch (error) {
    receipt.status = "failed";
    receipt.error =
      error instanceof Error ? { name: error.name, message: error.message } : String(error);
    throw error;
  } finally {
    write("setup.json.txt", receipt);
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  assert.equal(process.argv.length, 4, "WORKSPACE OUTPUT_DIRECTORY");
  await bootstrapMapperHost(string(process.argv[2]), string(process.argv[3]));
}
