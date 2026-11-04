// #11795: the Nitro 3 development hook must retain the stylesheet behavior.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, readJson } from "../../json-value.ts";
import { closeConsumer, freePort, loadChromium, type BrowserGlobals } from "./host-types.ts";
import { prepareNuxtSourceBinding } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const relative = "tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime";
const fixture = path.join(root, relative);
assert.ok(
  process.argv[2] && process.argv[3],
  "usage: nuxt5-dev-runtime-build.ts OUTPUT RUNTIME_NODE",
);
const output = path.resolve(process.argv[2]);
assert.equal(fs.existsSync(output), false, "retain a fresh packet");
fs.mkdirSync(output, { recursive: true });
const sha256 = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const write = (file: string, value: unknown) =>
  fs.writeFileSync(path.join(output, file), JSON.stringify(value, null, 2) + "\n");
assert.equal(process.execPath, process.env.VIZE_NUXT_SOURCE_NODE);
assert.equal(sha256(process.execPath), process.env.VIZE_NUXT_SOURCE_NODE_SHA256);
const node = fs.realpathSync(process.argv[3]);
const probe = spawnSync(
  node,
  [
    "--input-type=module",
    "--eval",
    "console.log(JSON.stringify({version:process.version,execPath:process.execPath,platform:process.platform,arch:process.arch}))",
  ],
  { encoding: "utf8" },
);
assert.equal(probe.error, undefined);
assert.equal(probe.status, 0);
const runtime: unknown = JSON.parse(probe.stdout);
assert.equal(at(runtime, "version"), "v26.13.0");
assert.equal(at(runtime, "execPath"), node);
assert.equal(at(runtime, "platform"), process.platform);
assert.equal(at(runtime, "arch"), process.arch);
const nodeHash = sha256(node);
write("runtime.json", {
  producer: {
    version: process.version,
    execPath: process.execPath,
    sha256: sha256(process.execPath),
  },
  consumer: runtime,
  sha256: nodeHash,
});
const pins = readJson(path.join(fixture, "provenance.json"));
const files = at(pins, "files");
assert.ok(files && typeof files === "object" && !Array.isArray(files));
for (const [file, hash] of Object.entries(files)) {
  assert.equal(sha256(path.join(fixture, file)), hash);
  fs.mkdirSync(path.dirname(path.join(output, "inputs", file)), { recursive: true });
  fs.copyFileSync(path.join(fixture, file), path.join(output, "inputs", file));
}
const versions: Record<string, unknown> = {};
for (const [name, version] of [
  ["nuxt", "5.0.0-2611031025-9496b7e"],
  ["nitro", "3.0.261029-rc"],
  ["vite", "8.3.5"],
  ["vue", "3.5.44"],
  ["@vizejs/nuxt", "0.676.0"],
  ["@vizejs/vite-plugin", "0.676.0"],
  ["@vizejs/native", "0.676.0"],
  ["vize", "0.676.0"],
] as const) {
  versions[name] = at(
    readJson(path.join(fixture, "node_modules", name, "package.json")),
    "version",
  );
  assert.equal(versions[name], version);
}
write("versions.json", versions);
const source = prepareNuxtSourceBinding(root, output, { fixtureDirectory: relative });
const swaps = [
  ["nuxt", "npm/framework/nuxt"],
  ["vite-plugin", "npm/builder/vite"],
] as const;
const swapped: string[] = [];
let server: ReturnType<typeof spawn> | undefined;
let serverError: Error | undefined;
let browser: Awaited<ReturnType<ReturnType<typeof loadChromium>["launch"]>> | undefined;
function hashes(directory: string): Record<string, string> {
  return Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort()
      .flatMap((name) =>
        fs.statSync(path.join(directory, name)).isFile()
          ? [[name, sha256(path.join(directory, name))]]
          : [],
      ),
  );
}
try {
  const dist: Record<string, Record<string, string>> = {};
  for (const [name, sourcePath] of swaps) {
    const installed = path.join(fixture, "node_modules/@vizejs", name, "dist");
    assert.ok(
      fs.realpathSync(installed).startsWith(fixture + path.sep),
      "use the fixture's private installation",
    );
    const candidate = path.join(root, sourcePath, "dist");
    dist[name] = hashes(candidate);
    fs.cpSync(installed, path.join(output, `published-${name}-dist`), { recursive: true });
    swapped.push(name);
    fs.rmSync(installed, { recursive: true });
    fs.cpSync(candidate, installed, { recursive: true });
    assert.deepEqual(hashes(installed), dist[name]);
  }
  write("candidate-dist.json", dist);
  for (const directory of [".nuxt", ".output", "node_modules/.cache"])
    fs.rmSync(path.join(fixture, directory), { recursive: true, force: true });
  const port = await freePort();
  const command = [
    node,
    "node_modules/nuxt/bin/nuxt.mjs",
    "dev",
    "--host",
    "127.0.0.1",
    "--port",
    String(port),
  ];
  const logFile = path.join(output, "dev.log");
  const log = fs.openSync(logFile, "w");
  server = spawn(node, command.slice(1), {
    cwd: fixture,
    env: {
      ...source.environment,
      PATH: `${path.dirname(node)}${path.delimiter}${process.env.PATH}`,
      NO_COLOR: "1",
      NUXT_TELEMETRY_DISABLED: "1",
    },
    stdio: ["ignore", log, log],
  });
  server.on("error", (error) => {
    serverError = error;
    write("server-error.json", { message: error.message });
  });
  assert.ok(server.pid, "the development server process started");
  fs.closeSync(log);
  write("server-process.json", { command, cwd: fixture, pid: server.pid, runtimeSha256: nodeHash });
  const url = `http://127.0.0.1:${port}/`;
  let html = "";
  for (let attempt = 0; attempt < 50; attempt++) {
    try {
      const response = await fetch(url, { signal: AbortSignal.timeout(2_000) });
      html = await response.text();
      write("ssr-response.json", {
        status: response.status,
        headers: Object.fromEntries(response.headers),
      });
      fs.writeFileSync(path.join(output, "ssr.html"), html);
      assert.equal(response.status, 200);
      break;
    } catch (error) {
      if (
        fs.existsSync(path.join(output, "ssr-response.json")) ||
        server.exitCode !== null ||
        serverError
      )
        throw error;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.match(html, /<button\b[^>]*>0<\/button>/);
  assert.equal((html.match(/ghost-vize-entry\.js/g) ?? []).length, 0);
  assert.equal((html.match(/stylesheet-control\.css\?direct/g) ?? []).length, 1);
  browser = await loadChromium(root).launch();
  const page = await browser.newPage();
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  page.on("console", (message) => {
    if (["warning", "error"].includes(message.type())) errors.push(message.text());
  });
  await page.goto(url, { waitUntil: "networkidle" });
  fs.writeFileSync(path.join(output, "browser-before.html"), await page.content());
  const styles = await page.getByRole("button", { name: "0" }).evaluate((button) => {
    const browser = globalThis as unknown as BrowserGlobals;
    const style = browser.getComputedStyle(button);
    return { color: style.color, outline: style.outlineColor };
  });
  write("styles.json", styles);
  assert.deepEqual(styles, { color: "rgb(17, 34, 51)", outline: "rgb(44, 55, 66)" });
  await page.getByRole("button", { name: "0" }).click();
  await page.getByRole("button", { name: "1" }).waitFor();
  fs.writeFileSync(path.join(output, "browser.html"), await page.content());
  write("browser-errors.json", errors);
  assert.deepEqual(errors, []);
  const logText = fs.readFileSync(logFile, "utf8");
  assert.doesNotMatch(logText, /NUXT_B9003|imports nitropack\/runtime/);
  assert.equal(sha256(node), nodeHash);
  const binding = source.verify();
  const events = fs.readFileSync(source.custody.calls, "utf8").trim().split("\n");
  for (const event of events) {
    const packet: unknown = JSON.parse(event);
    assert.equal(at(packet, "pid"), server.pid, "the observed Node 26 server compiled the SFC");
  }
  write("report.json", {
    passed: true,
    source: binding,
    dist,
    versions,
    consumerNode: { identity: runtime, sha256: nodeHash },
    counter: { before: 0, after: 1 },
    isolatedSsr: false,
    developmentSsr: 200,
    styles,
    ghostLinks: 0,
    validLinks: 1,
    nitroCompatibilityWarnings: 0,
  });
} catch (error) {
  fs.writeFileSync(
    path.join(output, "failure.txt"),
    error instanceof Error ? (error.stack ?? String(error)) : String(error),
  );
  throw error;
} finally {
  await closeConsumer(browser, server, () => {
    try {
      if (server) {
        write("server-exit.json", {
          code: server.exitCode,
          signal: server.signalCode,
          error: serverError?.message ?? null,
          controlledTermination: true,
        });
      }
    } finally {
      for (const name of swapped) {
        const installed = path.join(fixture, "node_modules/@vizejs", name, "dist");
        fs.rmSync(installed, { recursive: true });
        fs.cpSync(path.join(output, `published-${name}-dist`), installed, { recursive: true });
      }
    }
  });
}
