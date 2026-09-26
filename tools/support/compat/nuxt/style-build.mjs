// Real Nuxt #6825 regression: released adapter fails; candidate builds and serves scoped CSS.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tests/_fixtures/_projects/nuxt-scoped-style-build");
const candidate = path.join(root, "npm/builder/vite/dist");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-style-build"));
fs.mkdirSync(artifacts, { recursive: true });
const packageRoot = path.join(fixture, "node_modules/@vizejs/vite-plugin");
for (const [name, version] of [
  ["nuxt", "4.5.2"],
  ["vue", "3.5.43"],
  ["@vizejs/nuxt", "0.428.1"],
  ["@vizejs/native", "0.428.1"],
  ["@vizejs/vite-plugin", "0.428.1"],
]) {
  const actual = JSON.parse(
    fs.readFileSync(path.join(fixture, "node_modules", name, "package.json"), "utf8"),
  );
  assert.equal(actual.version, version, `fixture must use the exact ${name} version`);
}
assert.equal(
  fs.readFileSync(path.join(fixture, "nuxt.config.ts"), "utf8").includes("compiler: false"),
  false,
);
const published = path.join(artifacts, "published-vite-plugin-dist");
fs.cpSync(path.join(packageRoot, "dist"), published, { recursive: true });

function build(label) {
  for (const name of [".nuxt", ".output"])
    fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
  const log = path.join(artifacts, `${label}.log`);
  const fd = fs.openSync(log, "w");
  let result;
  try {
    result = spawnSync(
      process.execPath,
      [path.join(fixture, "node_modules/nuxt/bin/nuxt.mjs"), "build"],
      {
        cwd: fixture,
        env: { ...process.env, NO_COLOR: "1" },
        stdio: ["ignore", fd, fd],
        timeout: 240_000,
      },
    );
  } finally {
    fs.closeSync(fd);
  }
  assert.equal(result.error, undefined);
  return { status: result.status, output: fs.readFileSync(log, "utf8") };
}

try {
  const baseline = build("published-0.428.1");
  assert.notEqual(baseline.status, 0, "published adapter must reproduce the issue");
  assert.match(baseline.output, /lang=css(?:\.css){2,}/);
  assert.match(baseline.output, /Unknown word <\/style>/);
  console.log("Published 0.428.1 reproduced #6825 (compiler enabled)");

  fs.rmSync(path.join(packageRoot, "dist"), { recursive: true });
  fs.cpSync(candidate, path.join(packageRoot, "dist"), { recursive: true });
  const fixed = build("candidate");
  assert.equal(fixed.status, 0, `candidate Nuxt build failed; see ${artifacts}/candidate.log`);
  console.log("Candidate client and SSR builds passed (compiler enabled)");
  const portServer = net.createServer();
  await new Promise((resolve, reject) => {
    portServer.once("error", reject);
    portServer.listen(0, "127.0.0.1", resolve);
  });
  const port = portServer.address().port;
  await new Promise((resolve) => portServer.close(resolve));
  const serverLog = fs.openSync(path.join(artifacts, "server.log"), "w");
  const server = spawn(process.execPath, [path.join(fixture, ".output/server/index.mjs")], {
    cwd: fixture,
    env: { ...process.env, PORT: String(port), HOST: "127.0.0.1" },
    stdio: ["ignore", serverLog, serverLog],
  });
  const exited = new Promise((resolve) => server.once("exit", resolve));
  try {
    const origin = `http://127.0.0.1:${port}`;
    let response;
    for (let attempt = 0; attempt < 100; attempt++) {
      try {
        response = await fetch(origin, { signal: AbortSignal.timeout(1000) });
        break;
      } catch {
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
    }
    assert.ok(response, "Nitro server must become ready");
    assert.equal(response.status, 200);
    const html = await response.text();
    fs.writeFileSync(path.join(artifacts, "ssr.html"), html);
    const paragraph = html.match(/<p (data-v-[a-f\d]+)>hi<\/p>/);
    assert.ok(paragraph, "SSR must render the authored paragraph with its scope attribute");
    const stylesheets = [...html.matchAll(/<link\b[^>]*rel="stylesheet"[^>]*>/g)].map(
      ([tag]) => tag.match(/href="([^"]+)"/)[1],
    );
    assert.equal(stylesheets.length, 1);
    const cssResponse = await fetch(new URL(stylesheets[0], origin));
    assert.equal(cssResponse.status, 200);
    const css = await cssResponse.text();
    assert.equal(css.trim(), `p[${paragraph[1]}]{color:red}`);
    fs.writeFileSync(path.join(artifacts, "scoped.css"), css);
    fs.writeFileSync(
      path.join(artifacts, "proof.json"),
      JSON.stringify(
        {
          candidateHead: process.env.GITHUB_SHA ?? null,
          node: process.version,
          nuxt: "4.5.2",
          vue: "3.5.43",
          native: "0.428.1",
          baselineExit: baseline.status,
          candidateExit: fixed.status,
          scope: paragraph[1],
          stylesheets,
        },
        null,
        2,
      ) + "\n",
    );
    console.log(`SSR and linked CSS agree exactly: p[${paragraph[1]}]{color:red}`);
  } finally {
    server.kill("SIGTERM");
    await exited;
    fs.closeSync(serverLog);
  }
} finally {
  fs.rmSync(path.join(packageRoot, "dist"), { recursive: true, force: true });
  fs.cpSync(published, path.join(packageRoot, "dist"), { recursive: true });
}
