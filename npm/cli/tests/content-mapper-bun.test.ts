import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

function frame(value: unknown): string {
  const body = JSON.stringify(value);
  return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;
}

void test("Bun preserves negotiated content mapper capability arguments", async (context) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-mapper-bun-"));
  context.after(() => fs.rmSync(root, { force: true, recursive: true }));
  const packageRoot = path.join(root, "node_modules", "vize");
  const nativeRoot = path.join(packageRoot, "node_modules", "@vizejs", "native");
  fs.mkdirSync(nativeRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    JSON.stringify({ name: "vize", version: "0.0.0", type: "module" }),
  );
  fs.writeFileSync(
    path.join(nativeRoot, "package.json"),
    JSON.stringify({ name: "@vizejs/native", version: "0.0.0", main: "index.cjs" }),
  );
  fs.writeFileSync(
    path.join(nativeRoot, "index.cjs"),
    `const fs = require("node:fs");
exports.runCli = args => {
  const body = JSON.stringify({ id: 2, result: { args } });
  fs.writeSync(1, "Content-Length: " + Buffer.byteLength(body) + "\\r\\n\\r\\n" + body);
};
`,
  );
  fs.cpSync(fileURLToPath(new URL("../dist", import.meta.url)), path.join(packageRoot, "dist"), {
    recursive: true,
  });
  const cli = path.join(packageRoot, "bin", "vize");
  fs.mkdirSync(path.dirname(cli), { recursive: true });
  fs.copyFileSync(fileURLToPath(new URL("../bin/vize", import.meta.url)), cli);

  const child = spawn(process.execPath, [cli, "content-mapper"], { cwd: root });
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", (chunk: Buffer) => (stdout += chunk.toString()));
  child.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
  child.stdin.end(
    frame({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: {
        positionEncodings: ["utf-8"],
        capabilities: { diagnosticDirectiveCodeSelectors: true },
      },
    }),
  );
  const exitCode = await new Promise<number | null>((resolve) => child.on("exit", resolve));
  assert.equal(exitCode, 0, stderr);
  assert.equal(stderr, "");
  const bodies = [...stdout.matchAll(/Content-Length: (\d+)\r\n\r\n/g)].map((match, index, all) => {
    const start = (match.index ?? 0) + match[0].length;
    const next = all[index + 1]?.index ?? stdout.length;
    return JSON.parse(stdout.slice(start, next)) as Record<string, unknown>;
  });
  assert.deepEqual(bodies[0], {
    id: 1,
    jsonrpc: "2.0",
    result: { diagnosticSource: "vize", positionEncoding: "utf-8" },
  });
  assert.deepEqual((bodies[1].result as { args: string[] }).args, [
    "content-mapper",
    "--initialized",
    "--reply-when-ready",
    "--diagnostic-directive-code-selectors",
  ]);
});
