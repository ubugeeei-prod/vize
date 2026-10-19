// Exercise the candidate Vite adapter, source native compiler and CLI in a real
// Void (@void/vue) app: build, workerd SSR, dev SSR + HMR, check, lint, fmt.
import assert from "node:assert/strict";
import { type ChildProcess, spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { prepareNuxtSourceBinding } from "../nuxt/source-binding.mjs";

type Env = Record<string, string | undefined>;

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixtureDirectory = "tools/support/compat/void/fixtures/void-vue";
const fixture = path.join(root, fixtureDirectory);
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-void-vue"));
const vizeBin = path.resolve(process.argv[3] ?? path.join(root, "target/ci/vize"));
const requireTests = createRequire(path.join(root, "tests/package.json"));
fs.mkdirSync(artifacts, { recursive: true });
assert.ok(fs.existsSync(vizeBin), `candidate CLI ${vizeBin} must be built`);

for (const [name, version] of [
  ["void", "1.1.12"],
  ["@void/vue", "1.1.12"],
  ["vue", "3.5.44"],
  ["vite", "8.3.4"],
  ["@vizejs/vite-plugin", "0.474.0"],
]) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(fixture, "node_modules", name, "package.json"), "utf8"),
  );
  assert.equal(manifest.version, version, `fixture must use the exact ${name} version`);
}

function run(name: string, command: string, args: string[], env: Env = process.env) {
  const result = spawnSync(command, args, {
    cwd: fixture,
    env: { ...env, NO_COLOR: "1" },
    encoding: "utf8",
    timeout: 300_000,
  });
  fs.writeFileSync(path.join(artifacts, `${name}.log`), `${result.stdout}\n${result.stderr}`);
  assert.equal(result.error, undefined);
  return result;
}

function node(name: string, script: string, args: string[], env?: Env) {
  const result = run(name, process.execPath, [path.join(fixture, script), ...args], env);
  assert.equal(result.status, 0, `${name} failed; see ${name}.log`);
  return result;
}

async function freePort(): Promise<number> {
  const socket = net.createServer();
  await new Promise<void>((resolve, reject) => {
    socket.once("error", reject);
    socket.listen(0, "127.0.0.1", resolve);
  });
  const { port } = socket.address() as net.AddressInfo;
  await new Promise((resolve) => socket.close(resolve));
  return port;
}

async function serve(name: string, mode: "dev" | "preview", env: Env) {
  const port = await freePort();
  const log = fs.openSync(path.join(artifacts, `${name}.log`), "w");
  const server = spawn(
    process.execPath,
    ["node_modules/vite/bin/vite.js", mode, "--host", "127.0.0.1", "--port", String(port)],
    { cwd: fixture, env: { ...env, NO_COLOR: "1" }, stdio: ["ignore", log, log] },
  );
  const origin = `http://127.0.0.1:${port}`;
  for (let attempt = 0; attempt < 300; attempt++) {
    try {
      const response = await fetch(`${origin}/api/health`, { signal: AbortSignal.timeout(1000) });
      if (response.ok) return { origin, server };
    } catch {}
    if (server.exitCode !== null) break;
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  server.kill();
  throw new Error(`${name} server never answered; see ${name}.log`);
}

async function stop(server: ChildProcess) {
  if (server.exitCode !== null) return;
  const exited = new Promise((resolve) => server.once("exit", resolve));
  server.kill("SIGTERM");
  await exited;
}

async function page(origin: string, route: string): Promise<string> {
  const response = await fetch(origin + route);
  assert.equal(response.status, 200, `${route} returned ${response.status}`);
  const html = await response.text();
  fs.writeFileSync(path.join(artifacts, `${route.replaceAll("/", "_") || "_"}.html`), html);
  return html;
}

async function assertSsr(origin: string, built: boolean) {
  const home = await page(origin, "/");
  assert.match(home, /<h1 class="title" data-v-[a-f0-9]+>Void with Vize<\/h1>/);
  assert.match(home, /<button data-test="counter"[^>]*>Clicks: 2<\/button>/);
  if (built) {
    // Route CSS is looked up in the client manifest by the page's source path.
    const href = /<link rel="stylesheet" href="(\/assets\/[^"]+\.css)">/.exec(home)?.[1];
    assert.ok(href, "route stylesheet link missing from SSR HTML");
    assert.match(
      await (await fetch(origin + href)).text(),
      /\.title\[data-v-[a-f0-9]+\]\{color:#639\}/,
    );
  } else {
    assert.match(home, /\.title\[data-v-[a-f0-9]+\]\{color: rebeccapurple;\}/);
  }
  assert.match(await page(origin, "/users"), /<a href="\/users\/2"[^>]*>Grace<\/a>/);
  assert.match(await page(origin, "/users/2"), /<h1 data-test="user">Grace<\/h1>/);
  const visits = async () =>
    ((await (await fetch(`${origin}/api/visits`)).json()) as never)["visits"];
  assert.equal((await visits()) + 1, await visits(), "KV binding must persist across requests");
}

async function assertHydrationAndHmr(origin: string) {
  const { chromium } = requireTests("@playwright/test");
  const browser = await chromium.launch();
  const source = path.join(fixture, "pages/index.vue");
  const original = fs.readFileSync(source, "utf8");
  try {
    const tab = await browser.newPage();
    const errors: string[] = [];
    tab.on("pageerror", (error: Error) => errors.push(String(error)));
    let navigations = 0;
    tab.on("framenavigated", () => navigations++);
    await tab.goto(origin + "/");
    await tab.waitForSelector("#app[data-hydrated=true]");
    await tab.click("[data-test=counter]");
    await tab.waitForFunction(() => document.body.textContent?.includes("Clicks: 3"));
    const loaded = navigations;
    fs.writeFileSync(source, original.replace("Clicks:", "Taps:"));
    // A template edit must keep component state (template-only HMR).
    await tab.waitForFunction(
      () => document.querySelector("[data-test=counter]")?.textContent === "Taps: 3",
    );
    assert.equal(navigations, loaded, "HMR must not reload the page");
    assert.deepEqual(errors, []);
  } finally {
    fs.writeFileSync(source, original);
    await browser.close();
  }
}

type Located = { file: string; line: number };

function vueTscErrors(): Located[] {
  const result = run("vue-tsc", process.execPath, [
    "node_modules/vue-tsc/bin/vue-tsc.js",
    "--noEmit",
  ]);
  return [...`${result.stdout}`.matchAll(/^(\S+)\((\d+),\d+\): error TS\d+/gm)].map((match) => ({
    file: match[1],
    line: Number(match[2]),
  }));
}

function vizeErrors(): Located[] {
  const result = run("vize-check", vizeBin, ["check", "--format", "json"]);
  const report = JSON.parse(result.stdout.slice(result.stdout.indexOf("{")));
  // Each diagnostic is rendered as "error:LINE:COLUMN [TSxxxx] message".
  return (report.files as { file: string; diagnostics: string[] }[]).flatMap(
    ({ file, diagnostics }) =>
      diagnostics.flatMap((diagnostic) => {
        const match = /^error:(\d+):\d+ /.exec(diagnostic);
        return match ? [{ file, line: Number(match[1]) }] : [];
      }),
  );
}

const located = (errors: Located[]) => errors.map(({ file, line }) => `${file}:${line}`).sort();

function assertCheckParity() {
  assert.deepEqual(located(vizeErrors()), [], "vize check must accept the Void app");
  assert.deepEqual(located(vueTscErrors()), [], "vue-tsc must accept the Void app");
  // Seed what Void's generated types catch: loader props and typed routes.
  const seeds = [
    ["pages/users/[id].vue", "{{ user.name }}", "{{ user.nam }}"],
    ["pages/stats/index.island.vue", 'fetch("/api/visits")', 'fetch("/api/visit")'],
  ];
  const originals = seeds.map(([file]) => fs.readFileSync(path.join(fixture, file), "utf8"));
  try {
    seeds.forEach(([file, from, to], index) =>
      fs.writeFileSync(path.join(fixture, file), originals[index].replace(from, to)),
    );
    const expected = located(vueTscErrors());
    assert.ok(expected.length >= seeds.length, "vue-tsc must report the seeded errors");
    assert.deepEqual(located(vizeErrors()), expected, "vize check must match vue-tsc");
  } finally {
    seeds.forEach(([file], index) => fs.writeFileSync(path.join(fixture, file), originals[index]));
  }
}

const vitePlugin = path.join(fixture, "node_modules/@vizejs/vite-plugin/dist");
const published = path.join(artifacts, "published-vite-plugin-dist");
fs.cpSync(vitePlugin, published, { recursive: true });
fs.rmSync(vitePlugin, { recursive: true });
fs.cpSync(path.join(root, "npm/builder/vite/dist"), vitePlugin, { recursive: true });
const binding = prepareNuxtSourceBinding(root, artifacts, { fixtureDirectory });
try {
  // node_modules/.vize would replay earlier compiles instead of calling the source compiler.
  for (const name of ["dist", ".void", ".wrangler", "node_modules/.vize", "node_modules/.vite"])
    fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
  node("void-prepare", "node_modules/void/dist/cli/cli.mjs", ["prepare"], binding.environment);
  node("build", "node_modules/vite/bin/vite.js", ["build"], binding.environment);

  const preview = await serve("preview", "preview", binding.environment);
  try {
    await assertSsr(preview.origin, true);
  } finally {
    await stop(preview.server);
  }
  const dev = await serve("dev", "dev", binding.environment);
  try {
    await assertSsr(dev.origin, false);
    await assertHydrationAndHmr(dev.origin);
  } finally {
    await stop(dev.server);
  }
  const native = binding.verify();

  assertCheckParity();
  for (const [name, args] of [
    ["lint", ["lint"]],
    ["fmt", ["fmt", "--check", "pages", "routes", "env.ts", "vite.config.ts", "void.config.ts"]],
  ] as const) {
    const result = run(name, vizeBin, [...args]);
    assert.equal(result.status, 0, `vize ${name} failed on the Void app; see ${name}.log`);
  }
  fs.writeFileSync(path.join(artifacts, "proof.json"), JSON.stringify({ native }, null, 2) + "\n");
  console.log("Void + Vue: build, workerd SSR, dev SSR/HMR, check parity, lint and fmt verified");
} finally {
  fs.rmSync(vitePlugin, { recursive: true });
  fs.cpSync(published, vitePlugin, { recursive: true });
}
