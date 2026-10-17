import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createWriteStream } from "node:fs";
import { finished } from "node:stream/promises";
import { readFile, writeFile, mkdir, copyFile, readdir, unlink } from "node:fs/promises";
import path from "node:path";

export { assert, readFile, writeFile, mkdir, copyFile, readdir, unlink, path };
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export async function json(file, value) {
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`);
}
export async function run(command, args, directory, options = {}) {
  await mkdir(directory, { recursive: true });
  await json(path.join(directory, "command.json"), {
    command,
    args,
    cwd: process.cwd(),
    env: options.env ?? {},
  });
  const stdoutFile = path.join(directory, "stdout.bin");
  const stderrFile = path.join(directory, "stderr.bin");
  const stdout = createWriteStream(stdoutFile);
  const stderr = createWriteStream(stderrFile);
  const child = spawn(command, args, {
    env: { ...process.env, ...options.env },
    stdio: ["pipe", "pipe", "pipe"],
  });
  child.stdout.pipe(stdout);
  child.stderr.pipe(stderr);
  const input = options.input ?? Buffer.alloc(0);
  child.stdin.on("error", (error) => {
    if (error.code !== "EPIPE") child.kill();
  });
  child.stdin.end(input);
  const result = await new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", (status, signal) => resolve({ status, signal }));
  });
  await Promise.all([finished(stdout), finished(stderr)]);
  await json(path.join(directory, "outcome.json"), result);
  return { ...result, stdout: await readFile(stdoutFile), stderr: await readFile(stderrFile) };
}
export async function checked(command, args, directory, options) {
  const result = await run(command, args, directory, options);
  assert.equal(result.status, 0, `${command} ${args.join(" ")}; see ${directory}`);
  assert.equal(result.signal, null);
  return result.stdout.toString().trim();
}
export async function inventory(root) {
  const entries = [];
  async function visit(directory) {
    for (const item of (await readdir(directory, { withFileTypes: true })).sort((a, b) =>
      a.name.localeCompare(b.name),
    )) {
      const file = path.join(directory, item.name);
      if (item.isDirectory()) await visit(file);
      else if (item.isFile())
        entries.push({ file: path.relative(root, file), sha256: sha256(await readFile(file)) });
      else assert.fail(`unexpected artifact entry ${file}`);
    }
  }
  await visit(root);
  return entries;
}
export function frames(requests) {
  return Buffer.concat(
    requests.flatMap((request) => {
      const body = Buffer.from(JSON.stringify(request));
      return [Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body];
    }),
  );
}
export function parseFrames(bytes) {
  const packets = [];
  let offset = 0;
  while (offset < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", offset);
    assert(end >= offset, "complete header required");
    const header = bytes.subarray(offset, end).toString();
    assert.match(header, /^Content-Length: [0-9]+$/);
    const length = Number(header.slice("Content-Length: ".length));
    offset = end + 4;
    assert(offset + length <= bytes.length, "complete body required");
    packets.push(JSON.parse(bytes.subarray(offset, offset + length).toString()));
    offset += length;
  }
  return packets;
}
export async function transforms(cli, phase) {
  const requests = [
    { jsonrpc: "2.0", id: 1, method: "initialize", params: { positionEncodings: ["utf-8"] } },
    {
      jsonrpc: "2.0",
      id: 2,
      method: "openProject",
      params: { projectHandle: "owners", configFileName: "", compilerOptions: { strict: true } },
    },
  ];
  for (let index = 0; index < 16; index++) {
    const content = await readFile(
      path.join(phase, "packets/cli", `case-${index}/Owner.vue`),
      "utf8",
    );
    requests.push({
      jsonrpc: "2.0",
      id: index + 3,
      method: "transform",
      params: {
        projectHandle: "owners",
        fileName: path.resolve("Owner.vue"),
        content,
      },
    });
  }
  requests.push({
    jsonrpc: "2.0",
    id: 19,
    method: "closeProject",
    params: { projectHandle: "owners" },
  });
  const input = frames(requests);
  const directory = path.join(phase, "mapper");
  const output = await run(cli, ["content-mapper"], directory, { input });
  await writeFile(path.join(directory, "stdin.bin"), input);
  const packets = parseFrames(output.stdout);
  await json(path.join(directory, "responses.json"), packets);
  assert.equal(output.status, 0);
  assert.equal(output.signal, null);
  assert.equal(output.stderr.length, 0);
  assert.equal(packets.length, 19);
  packets.forEach((packet, index) => {
    assert.equal(packet.id, index + 1);
    assert(!Object.hasOwn(packet, "error"));
  });
  return packets.slice(2, 18).map((packet) => packet.result);
}
