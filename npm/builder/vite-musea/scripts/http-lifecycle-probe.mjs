import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const project = path.join(root, "npm/builder/vite-musea");
const relative = "npm/builder/vite-musea/src/variant-bindings-http.test.ts";
const baseline = process.argv[2];
assert.match(baseline, /^[a-f0-9]{40}$/);
const output = path.resolve(process.argv[3]);
mkdirSync(output, { recursive: true });
const sha256 = (body) => createHash("sha256").update(body).digest("hex");
const before = execFileSync("git", ["show", `${baseline}:${relative}`], {
  cwd: root,
  encoding: "utf8",
});
const after = readFileSync(path.join(root, relative), "utf8");
const preload = fileURLToPath(new URL("./http-lifecycle-preload.mjs", import.meta.url));
const tsx = createRequire(path.join(project, "package.json")).resolve("tsx/cli");
const receipt = {
  baseline,
  source: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
  node: process.version,
  executable: process.execPath,
  observations: [],
};

function instrument(source) {
  const markers = [
    [
      "  const server = await createServer({",
      '  mark("create:start");\n  const server = await createServer({',
    ],
    [
      "  const http = createHttpServer(server.middlewares);",
      '  mark("create:finished", {root: server.config.root, cacheDir: server.config.cacheDir});\n  const http = createHttpServer(server.middlewares);',
    ],
    ["  t.after(async () => {", '  t.after(async () => {\n    mark("cleanup:start");'],
    [
      "    await server.close();",
      '    mark("vite-close:start");\n    await server.close();\n    mark("vite-close:finished");',
    ],
    [
      "  assert.equal(missing.status, 400);",
      '  assert.equal(missing.status, 400);\n  mark("requests:verified");',
    ],
  ];
  for (const [needle, replacement] of markers) {
    assert.equal(source.split(needle).length, 2, `phase marker authority: ${needle}`);
    source = source.replace(needle, replacement);
  }
  return (
    `import { appendFileSync as appendPhase } from "node:fs";\n` +
    `const mark = (phase, details = {}) => appendPhase(process.env.VIZE_MUSEA_HTTP_DIAGNOSTICS + "/phases-" + process.pid + ".jsonl", JSON.stringify({phase, pid: process.pid, sharedObjects: process.report.getReport().sharedObjects, ...details}) + "\\n");\n` +
    source
  );
}

for (const [name, source] of [
  ["baseline", before],
  ["isolated", after],
]) {
  for (let trial = 1; trial <= 2; trial++) {
    const directory = path.join(output, `${name}-${trial}`);
    mkdirSync(directory);
    writeFileSync(path.join(directory, "authored.test.ts"), source);
    const filename = path.join(project, "src", `.http-lifecycle-${name}-${trial}.test.ts`);
    const transformed = instrument(source);
    writeFileSync(filename, transformed, { flag: "wx" });
    const args = ["--import", preload, tsx, "--test", "--test-reporter=tap", filename];
    const env = {
      ...process.env,
      DEBUG: "vite:deps*",
      NODE_OPTIONS: `${process.env.NODE_OPTIONS ?? ""} --import=${pathToFileURL(preload).href}`,
      VIZE_MUSEA_HTTP_DIAGNOSTICS: directory,
    };
    delete env.NODE_TEST_CONTEXT;
    let result;
    try {
      writeFileSync(path.join(directory, "observed.test.ts"), transformed);
      result = spawnSync(process.execPath, args, {
        cwd: project,
        env,
        encoding: "utf8",
        timeout: 60_000,
        maxBuffer: 16 * 1024 * 1024,
      });
    } finally {
      rmSync(filename, { force: true });
    }
    writeFileSync(path.join(directory, "stdout.txt"), result.stdout ?? "");
    writeFileSync(path.join(directory, "stderr.txt"), result.stderr ?? "");
    receipt.observations.push({
      name,
      trial,
      sourceSha256: sha256(source),
      observedSha256: sha256(transformed),
      argv: [process.execPath, ...args],
      status: result.status,
      signal: result.signal,
      error: result.error?.message,
      directory,
    });
    writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  }
}
console.log(JSON.stringify(receipt, null, 2));
// Preserve every observation before failing on any real signal, assertion or timeout.
assert.ok(
  receipt.observations.every((item) => item.status === 0 && item.signal === null && !item.error),
  "HTTP lifecycle diagnostic observed a real failure; retained complete evidence",
);
