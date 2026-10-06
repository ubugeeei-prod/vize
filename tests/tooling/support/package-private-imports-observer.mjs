// Passive custody around the byte-exact reported 20-second client.
import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import path from "node:path";
const originalSpawn = childProcess.spawn;
childProcess.spawn = function (command, argv, options) {
  assert.equal(command, path.resolve("node_modules/.bin/vize"));
  assert.deepEqual(argv, ["lsp", "--stdio"]);
  assert.deepEqual(options, { cwd: process.argv[2], stdio: ["pipe", "pipe", "ignore"] });
  const child = originalSpawn.call(this, command, argv, options);
  const client = [];
  const server = [];
  const originalWrite = child.stdin.write;
  child.stdin.write = function (bytes, ...rest) {
    client.push(Buffer.from(bytes));
    return originalWrite.call(this, bytes, ...rest);
  };
  child.stdout.on("data", (bytes) => server.push(Buffer.from(bytes)));
  child.on("close", (status, signal) => {
    const clientBytes = Buffer.concat(client);
    const serverBytes = Buffer.concat(server);
    const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
    const output = process.env.VIZE_PRIVATE_IMPORT_ORIGINAL_CAPTURE;
    assert.ok(output);
    fs.writeFileSync(
      output,
      JSON.stringify(
        {
          clientPid: process.pid,
          childPid: child.pid,
          command,
          argv,
          options,
          clientWireBase64: clientBytes.toString("base64"),
          serverWireBase64: serverBytes.toString("base64"),
          clientWireSha256: digest(clientBytes),
          serverWireSha256: digest(serverBytes),
          status,
          signal,
          sourceSha: process.env.SOURCE_SHA,
          stderrRepresentation: "original reported client intentionally ignores server stderr",
          terminalRepresentation: "child close after stdout drained",
        },
        null,
        2,
      ) + "\n",
    );
  });
  return child;
};
syncBuiltinESMExports();
