import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

// This is a runtime acceptance check, deliberately invoked only after setup-deno.
// The fixture serves immutable GET responses; it never writes to a registry.
const source = fileURLToPath(new URL("../../../../", import.meta.url));
const output = resolve(process.argv[2] ?? "artifacts/jsr-consumer-age");
mkdirSync(output, { recursive: true });
const root = realpathSync(mkdtempSync(join(tmpdir(), "vize-jsr-consumer-age-")));
const policy = readFileSync(join(source, "tools/support/release/jsr/consumer-deno.json"), "utf8");
const evidence = {
  schema: "vize.jsr.consumer-age",
  proof: "controlled-registry",
  status: "running",
  sourceSha: execFileSync("git", ["rev-parse", "HEAD"], { cwd: source, encoding: "utf8" }).trim(),
  runId: process.env.GITHUB_RUN_ID ?? null,
  attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
  denoVersion: execFileSync("deno", ["--version"], { encoding: "utf8" }).trim(),
  policy: JSON.parse(policy),
  policySha256: createHash("sha256").update(policy).digest("hex"),
  consumerSha256: createHash("sha256")
    .update(readFileSync(join(source, "tools/support/release/jsr/consumer.ts")))
    .digest("hex"),
  fixtures: [],
  commands: [],
  requests: [],
};
const save = () =>
  writeFileSync(join(output, `${evidence.proof}.json`), `${JSON.stringify(evidence, null, 2)}\n`);
save();
assert.match(evidence.denoVersion, /^deno 2\.9\.7\s/);

async function run(label, args, cwd, extraEnv = {}) {
  const cache = join(root, `${label}-cache`);
  mkdirSync(cache);
  const command = {
    label,
    argv: ["deno", ...args],
    cwd,
    denoDir: cache,
    env: extraEnv,
    exitCode: null,
    stdout: "",
    stderr: "",
  };
  evidence.commands.push(command);
  await new Promise((resolveRun, reject) => {
    const child = spawn("deno", args, {
      cwd,
      env: { ...process.env, ...extraEnv, DENO_DIR: cache },
    });
    child.stdout.setEncoding("utf8").on("data", (chunk) => {
      command.stdout += chunk;
    });
    child.stderr.setEncoding("utf8").on("data", (chunk) => {
      command.stderr += chunk;
    });
    child.once("error", reject);
    child.once("close", (code) => {
      command.exitCode = code;
      resolveRun();
    });
  });
  save();
  return command;
}

const packages = new Map();
const server = createServer((request, response) => {
  evidence.requests.push({ method: request.method, url: request.url });
  if (request.method !== "GET") {
    response.writeHead(405).end();
    return;
  }
  const key = decodeURIComponent(request.url.split("?")[0].slice(1));
  const value = packages.get(key);
  if (!value) {
    response.writeHead(404).end();
    return;
  }
  response.setHeader(
    "Content-Type",
    Buffer.isBuffer(value) ? "application/octet-stream" : "application/json",
  );
  response.end(Buffer.isBuffer(value) ? value : JSON.stringify(value));
});
await new Promise((resolveListen) => server.listen(0, "127.0.0.1", resolveListen));
const registry = `http://127.0.0.1:${server.address().port}/`;

try {
  const now = Date.now();
  for (const [name, ageHours] of [
    ["@vizejs/native", 1],
    ["vize-age-external-control", 1],
    ["vize-age-established-control", 48],
  ]) {
    const directory = join(root, name.replaceAll("/", "-"));
    const content = join(directory, "package");
    mkdirSync(content, { recursive: true });
    const manifest = { name, version: "1.0.0", type: "module", main: "index.js" };
    writeFileSync(join(content, "package.json"), JSON.stringify(manifest));
    writeFileSync(join(content, "index.js"), 'export const marker = "age-control-executed";\n');
    const archive = join(directory, "fixture.tgz");
    execFileSync("tar", [
      "-czf",
      archive,
      "-C",
      directory,
      "package/package.json",
      "package/index.js",
    ]);
    const bytes = readFileSync(archive);
    const tarball = `${name.replaceAll("/", "-")}.tgz`;
    evidence.fixtures.push({
      name,
      version: "1.0.0",
      publishedAt: new Date(now - ageHours * 3600000).toISOString(),
      archiveSha256: createHash("sha256").update(bytes).digest("hex"),
    });
    packages.set(tarball, bytes);
    packages.set(name, {
      name,
      "dist-tags": { latest: "1.0.0" },
      time: { "1.0.0": new Date(now - ageHours * 3600000).toISOString() },
      versions: {
        "1.0.0": {
          ...manifest,
          dist: {
            tarball: `${registry}${tarball}`,
            integrity: `sha512-${createHash("sha512").update(bytes).digest("base64")}`,
          },
        },
      },
    });
    const consumer = join(directory, "consumer");
    mkdirSync(consumer);
    writeFileSync(
      join(consumer, "package.json"),
      '{"name":"vize-jsr-consumer","private":true,"type":"module"}\n',
    );
    writeFileSync(join(consumer, "deno.json"), policy);
    writeFileSync(
      join(consumer, "consumer.ts"),
      `const { marker } = await import(${JSON.stringify(`npm:${name}@1.0.0`)}); if (marker !== "age-control-executed") throw new Error("control did not execute"); console.log(marker);\n`,
    );
    const result = await run(
      name.replaceAll("/", "-"),
      ["run", "--config", "deno.json", "--allow-all", "consumer.ts"],
      consumer,
      { NPM_CONFIG_REGISTRY: registry },
    );
    if (name === "vize-age-external-control") {
      assert.notEqual(
        result.exitCode,
        0,
        "fresh external dependency bypassed the 24-hour age policy",
      );
      assert.match(result.stderr, /minimum dependency date|minimum dependency age|newer than/i);
      assert.ok(!result.stdout.includes("age-control-executed"));
      assert.ok(
        !evidence.requests.some((request) => request.url.includes(tarball)),
        "rejected dependency archive was fetched",
      );
    } else {
      assert.equal(result.exitCode, 0, result.stderr);
      assert.match(result.stdout, /age-control-executed/);
    }
  }
  evidence.controlledRegistry =
    "passed: fresh owned allowed, fresh external rejected, established external allowed";

  evidence.status = "passed";
  save();
  const version = process.env.JSR_CONSUMER_PUBLISHED_VERSION;
  if (version) {
    evidence.proof = "public-registry";
    evidence.status = "running";
    evidence.commands = [];
    evidence.requests = [];
    evidence.fixtures = [];
    delete evidence.controlledRegistry;
    assert.match(version, /^0\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/);
    const consumer = join(root, "published-consumer");
    mkdirSync(consumer);
    copyFileSync(
      join(source, "tools/support/release/jsr/consumer.ts"),
      join(consumer, "consumer.ts"),
    );
    writeFileSync(
      join(consumer, "package.json"),
      '{"name":"vize-jsr-consumer","private":true,"type":"module"}\n',
    );
    writeFileSync(join(consumer, "deno.json"), policy);
    const result = await run(
      "public-registry",
      ["run", "--config", "deno.json", "--allow-all", "consumer.ts", `jsr:@vizejs/vize@${version}`],
      consumer,
    );
    assert.equal(result.exitCode, 0, result.stderr);
    assert.ok(result.stdout.includes(`Verified jsr:@vizejs/vize@${version}`));
    evidence.status = "passed";
    evidence.publicRegistry = {
      version,
      status: "passed",
      scope:
        "actual published facade, config and native compiler consumer; separate from controlled-registry age semantics",
    };
  }
  console.log("Deno consumer age controls passed.");
} catch (error) {
  evidence.status = "failed";
  evidence.error = error.message;
  throw error;
} finally {
  save();
  await new Promise((resolveClose) => server.close(resolveClose));
  rmSync(root, { recursive: true, force: true });
}
