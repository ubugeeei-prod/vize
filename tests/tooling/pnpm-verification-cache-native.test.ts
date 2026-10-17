import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { once } from "node:events";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { promisify } from "node:util";
import { readRepoFile } from "./support/github-workflows.ts";

const execute = promisify(execFile);
const packageManager: string = JSON.parse(readRepoFile("package.json")).packageManager;
const name = "vize-cache-policy-control";
const integrity = `sha512-${Buffer.alloc(64).toString("base64")}`;

// A real frozen lockfile-only install exercises pnpm's policy gate without
// fetching a tarball, building packages, or accessing the public registry.
test("pinned pnpm reuses only its valid policy verdict and rejects changed policy or lockfile", async (t) => {
  const directory = mkdtempSync(join(tmpdir(), "vize-pnpm-policy-native-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const requests: string[] = [];
  const server = createServer((request, response) => {
    requests.push(request.url ?? "");
    if (request.url !== `/${name}`) {
      response.writeHead(404);
      response.end();
      return;
    }
    response.setHeader("Content-Type", "application/json");
    response.end(
      JSON.stringify({
        name,
        "dist-tags": { latest: "1.0.0" },
        time: {
          created: "2020-01-01T00:00:00.000Z",
          modified: "2020-01-01T00:00:00.000Z",
          "0.9.0": "2019-01-01T00:00:00.000Z",
          "1.0.0": "2020-01-01T00:00:00.000Z",
        },
        versions: {
          "0.9.0": {
            name,
            version: "0.9.0",
            _npmUser: {
              name: "publisher",
              trustedPublisher: { id: "github", oidcConfigId: "release" },
            },
            dist: {
              integrity,
              tarball: `${registry}${name}/-/${name}-0.9.0.tgz`,
              attestations: { provenance: { predicateType: "https://slsa.dev/provenance/v1" } },
            },
          },
          "1.0.0": {
            name,
            version: "1.0.0",
            dist: {
              integrity,
              tarball: `${registry}${name}/-/${name}-1.0.0.tgz`,
              attestations: { provenance: { predicateType: "https://slsa.dev/provenance/v1" } },
            },
          },
        },
      }),
    );
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(() => server.close());
  const registry = `http://127.0.0.1:${(server.address() as AddressInfo).port}/`;
  const producer = join(directory, "producer");
  const consumer = join(directory, "consumer");
  for (const root of [producer, consumer]) {
    mkdirSync(root);
    writeFileSync(
      join(root, "package.json"),
      JSON.stringify({ private: true, dependencies: { [name]: "1.0.0" } }),
    );
    writeFileSync(
      join(root, "pnpm-workspace.yaml"),
      `registry: ${registry}\nminimumReleaseAge: 1440\n`,
    );
    writeFileSync(
      join(root, "pnpm-lock.yaml"),
      `lockfileVersion: '9.0'
importers:
  .:
    dependencies:
      ${name}:
        specifier: 1.0.0
        version: 1.0.0
packages:
  ${name}@1.0.0:
    resolution: {integrity: ${integrity}}
snapshots:
  ${name}@1.0.0: {}
`,
    );
  }
  const environment = {
    ...Object.fromEntries(
      Object.entries(process.env).filter(([key]) => !/^(?:pnpm|npm)_config_/i.test(key)),
    ),
    COREPACK_ENABLE_NETWORK: "0",
  };
  // vp run/exec selects the managed pnpm on PATH, but also exposes the workspace
  // vp binary, which does not implement the global CLI's `env` subcommand.
  const version = await execute("pnpm", ["--version"], {
    cwd: producer,
    env: environment,
    timeout: 15_000,
  });
  assert.equal(
    `pnpm@${version.stdout.trim()}`,
    packageManager,
    "native verification controls require the exact pinned pnpm runtime",
  );
  const install = async (root: string, age = "1440", trustPolicy = "off") => {
    requests.length = 0;
    const options = {
      cwd: root,
      env: {
        ...environment,
        CI: "true",
        PNPM_CONFIG_CACHE_DIR: join(root, "cache"),
        PNPM_CONFIG_STORE_DIR: join(root, "store"),
        PNPM_CONFIG_REGISTRY: registry,
        PNPM_CONFIG_MINIMUM_RELEASE_AGE: age,
        PNPM_CONFIG_TRUST_POLICY: trustPolicy,
        PNPM_CONFIG_UPDATE_NOTIFIER: "false",
      },
      timeout: 15_000,
      maxBuffer: 1024 * 1024,
    };
    let code = 0;
    let output: string;
    try {
      const result = await execute(
        "pnpm",
        [
          "install",
          "--frozen-lockfile",
          "--lockfile-only",
          "--ignore-scripts",
          "--reporter=ndjson",
        ],
        options,
      );
      output = result.stdout + result.stderr;
    } catch (error) {
      const result = error as { code?: number; stdout?: string; stderr?: string };
      if (result.code !== 1) throw error;
      code = result.code;
      output = (result.stdout ?? "") + (result.stderr ?? "");
    }
    const events = output.split("\n").flatMap((line) => {
      try {
        const event = JSON.parse(line);
        return event.name === "pnpm:lockfile-verification" ? [event.status] : [];
      } catch {
        return [];
      }
    });
    return { code, output, events, requests: [...requests] };
  };

  const cold = await install(producer);
  assert.equal(cold.code, 0, cold.output);
  assert.deepEqual(cold.events, ["started", "done"]);
  assert.deepEqual(cold.requests, [`/${name}`]);
  const receipt = "lockfile-verified.jsonl";
  mkdirSync(join(consumer, "cache"));
  copyFileSync(join(producer, "cache", receipt), join(consumer, "cache", receipt));
  const warm = await install(consumer);
  assert.equal(warm.code, 0, warm.output);
  assert.deepEqual(warm.events, ["cached"]);
  assert.deepEqual(warm.requests, []);

  const verified = readFileSync(join(consumer, "cache", receipt), "utf8");
  const provenance = await install(consumer, "1440", "no-downgrade");
  assert.equal(provenance.code, 1, provenance.output);
  assert.deepEqual(provenance.events, ["started", "failed"]);
  assert.match(provenance.output, /ERR_PNPM_TRUST_DOWNGRADE/);
  assert.equal(readFileSync(join(consumer, "cache", receipt), "utf8"), verified);

  const stricter = await install(consumer, "52560000");
  assert.equal(stricter.code, 1, stricter.output);
  assert.deepEqual(stricter.events, ["started", "failed"]);
  assert.match(stricter.output, /ERR_PNPM_MINIMUM_RELEASE_AGE_VIOLATION/);
  assert.equal(readFileSync(join(consumer, "cache", receipt), "utf8"), verified);

  const lockfile = join(consumer, "pnpm-lock.yaml");
  const validLockfile = readFileSync(lockfile, "utf8");
  writeFileSync(
    lockfile,
    validLockfile.replace(
      `integrity: ${integrity}`,
      `tarball: ${registry}${name}/-/${name}-1.0.0.tgz`,
    ),
  );
  const missingIntegrity = await install(consumer);
  assert.equal(missingIntegrity.code, 1, missingIntegrity.output);
  assert.deepEqual(missingIntegrity.events, ["started", "failed"]);
  assert.deepEqual(missingIntegrity.requests, []);
  assert.match(missingIntegrity.output, /ERR_PNPM_MISSING_TARBALL_INTEGRITY/);
  assert.equal(readFileSync(join(consumer, "cache", receipt), "utf8"), verified);

  writeFileSync(lockfile, validLockfile);
  writeFileSync(join(consumer, "cache", receipt), "corrupt cache record\n");
  const repaired = await install(consumer);
  assert.equal(repaired.code, 0, repaired.output);
  assert.deepEqual(repaired.events, ["started", "done"]);
  assert.deepEqual(repaired.requests, [`/${name}`]);
});
