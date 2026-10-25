// The original reported consumer stays byte-for-byte intact; only its installed
// adapter link changes between the published failure and the source candidate.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const args = process.argv.slice(2);
assert.deepEqual([args[0], args[2], args[4]], ["--project", "--candidate-root", "--output"]);
assert.equal(args.length, 6);
const project = fs.realpathSync(args[1]);
const candidate = fs.realpathSync(args[3]);
const output = path.resolve(args[5]);
assert.ok(!fs.existsSync(output), "retain previous failed evidence; use a fresh output directory");
fs.mkdirSync(output, { recursive: true });
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const json = (file) => JSON.parse(fs.readFileSync(file, "utf8"));
const packageLink = path.join(project, "node_modules/@vizejs/vite-plugin");
const published = fs.realpathSync(packageLink);
assert.equal(json(path.join(published, "package.json")).version, "0.569.0");
assert.equal(json(path.join(project, "node_modules/vue/package.json")).version, "3.5.44");
assert.equal(json(path.join(project, "node_modules/vite/package.json")).version, "8.3.5");
assert.equal(json(path.join(candidate, "package.json")).name, "@vizejs/vite-plugin");
assert.ok(fs.existsSync(path.join(candidate, "dist/index.mjs")), "build the genuine adapter first");

function processPacket(command, argv, cwd) {
  const result = spawnSync(command, argv, {
    cwd,
    env: process.env,
    encoding: "utf8",
    timeout: 180_000,
  });
  return {
    command,
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    stdout: result.stdout,
    stderr: result.stderr,
    error: result.error?.message ?? null,
  };
}

const upstream = processPacket("git", ["rev-parse", "HEAD"], project);
assert.equal(upstream.status, 0);
assert.equal(upstream.stdout.trim(), "9df512b6be97428d37ca4e4d769c33b2babd512e");
const clean = processPacket("git", ["diff", "--exit-code", "HEAD"], project);
assert.equal(clean.status, 0, "all authored reproduction files and lockfile must stay unchanged");
const files = processPacket("git", ["ls-files", "-z"], project).stdout.split("\0").filter(Boolean);
const inputs = files.map((file) => {
  const bytes = fs.readFileSync(path.join(project, file));
  const destination = path.join(output, "upstream", file);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, bytes);
  return { file, sha256: hash(bytes) };
});

function provider(packageRoot) {
  const worker = `const fs=require('fs'),crypto=require('crypto'),{createRequire}=require('module');
const req=createRequire(process.argv[1]+'/package.json');const entry=req.resolve('@vizejs/native');req('@vizejs/native');
const images=Object.keys(require.cache).filter(file=>file.endsWith('.node')).map(file=>({file:fs.realpathSync(file),sha256:crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')}));
process.stdout.write(JSON.stringify({entry:fs.realpathSync(entry),images}));`;
  const packet = processPacket(process.execPath, ["-e", worker, packageRoot], project);
  assert.equal(packet.status, 0, JSON.stringify(packet));
  assert.equal(packet.stderr, "", JSON.stringify(packet));
  return { packet, ...JSON.parse(packet.stdout) };
}

const providers = { published: provider(published), candidate: provider(candidate) };
if (process.env.VIZE_STYLE_REQUIRE_SOURCE_NATIVE === "1") {
  const preparation = json(
    path.join(repo, "npm/native/.artifacts/native/js-test-preparation.json"),
  );
  const binding = fs.realpathSync(path.join(repo, "npm/native", preparation.binding));
  assert.equal(providers.candidate.entry, fs.realpathSync(path.join(repo, "npm/native/index.js")));
  assert.equal(preparation.head, process.env.SOURCE_SHA);
  assert.equal(hash(fs.readFileSync(binding)), preparation.sha256);
  assert.ok(
    providers.candidate.images.some(
      (image) => image.file === binding && image.sha256 === preparation.sha256,
    ),
    "the physical current-source addon must load; fallback is forbidden",
  );
  fs.writeFileSync(
    path.join(output, "source-native-preparation.json"),
    JSON.stringify(preparation, null, 2),
  );
}
const viteRoot = fs.realpathSync(path.join(project, "node_modules/vite"));
const cli = path.join(viteRoot, json(path.join(viteRoot, "package.json")).bin.vite);
assert.ok(fs.existsSync(cli));
const lightning = createRequire(path.join(viteRoot, "package.json")).resolve("lightningcss");
assert.equal(json(path.resolve(path.dirname(lightning), "../package.json")).version, "1.33.0");
const config = path.join(project, "vite.style-lightning.config.mjs");
assert.ok(!fs.existsSync(config));
fs.writeFileSync(
  config,
  'import config from "./vite.config.js";\nexport default {...config,css:{transformer:"lightningcss"}};\n',
);
const observations = [];
const linkTarget = fs.readlinkSync(packageLink);

function collect(directory) {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((item) => {
      const file = path.join(directory, item.name);
      if (item.isDirectory()) return collect(file);
      const bytes = fs.readFileSync(file);
      return [
        {
          file: path.relative(path.join(project, "dist"), file),
          sha256: hash(bytes),
          bytes: bytes.toString("base64"),
        },
      ];
    })
    .sort((left, right) => left.file.localeCompare(right.file));
}

function build(label, backend, cold) {
  fs.rmSync(path.join(project, "dist"), { recursive: true, force: true });
  if (cold)
    fs.rmSync(path.join(project, "node_modules/.vize/vite-precompile"), {
      recursive: true,
      force: true,
    });
  const argv = [cli, "build", ...(backend === "lightningcss" ? ["--config", config] : [])];
  const packet = processPacket(process.execPath, argv, project);
  const artifacts = packet.status === 0 ? collect(path.join(project, "dist")) : [];
  const observation = { label, backend, cold, packet, artifacts };
  observations.push(observation);
  fs.writeFileSync(path.join(output, label + ".json"), JSON.stringify(observation, null, 2));
  return observation;
}

try {
  for (const backend of ["postcss", "lightningcss"]) {
    const before = build("published-0.569-" + backend, backend, true);
    assert.notEqual(
      before.packet.status,
      0,
      "the genuine published package must distinguish the issue",
    );
    assert.match(before.packet.stdout + before.packet.stderr, /ENOENT/);
    assert.match(before.packet.stdout + before.packet.stderr, /\.\.\/styles\/colors\.css/);
  }
  fs.unlinkSync(packageLink);
  fs.symlinkSync(candidate, packageLink, "junction");
  for (const backend of ["postcss", "lightningcss"]) {
    const cold = build("candidate-cold-" + backend, backend, true);
    const warm = build("candidate-warm-" + backend, backend, false);
    for (const result of [cold, warm]) {
      assert.equal(result.packet.status, 0, JSON.stringify(result.packet));
      const styles = result.artifacts.filter((item) => item.file.endsWith(".css"));
      assert.equal(styles.length, 1);
      assert.equal(Buffer.from(styles[0].bytes, "base64").toString(), ".card{color:tomato}\n");
      for (const artifact of result.artifacts) {
        const body = Buffer.from(artifact.bytes, "base64").toString();
        for (const origin of [project, candidate, published])
          assert.ok(
            !body.includes(origin),
            "physical resolution paths must not leak into shipped assets",
          );
      }
    }
    assert.deepEqual(
      cold.artifacts,
      warm.artifacts,
      "cold and warm assets must remain byte-identical",
    );
  }
} finally {
  fs.unlinkSync(packageLink);
  fs.symlinkSync(linkTarget, packageLink, "junction");
  fs.rmSync(config, { force: true });
  fs.writeFileSync(
    path.join(output, "receipt.json"),
    JSON.stringify(
      {
        source: process.env.SOURCE_SHA ?? null,
        node: process.version,
        upstream,
        inputs,
        candidate,
        published,
        providers,
        lightning: { entry: lightning, sha256: hash(fs.readFileSync(lightning)) },
        observations,
      },
      null,
      2,
    ),
  );
}
