// Observe an actual CLI without generating, changing, or consuming protocol packets.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const { spawn } = require("node:child_process");

const binary = process.env.VIZE_TEST_OBSERVED_SERVER_PATH;
const evidence = process.env.VIZE_TEST_CONFIG_PROTOCOL_PATH;
if (!binary || !path.isAbsolute(binary) || !evidence) {
  throw new Error("The passive server requires an absolute observed binary and evidence path");
}
const argv = process.argv.slice(2);
const record = (event) => fs.appendFileSync(evidence, `${JSON.stringify(event)}\n`);
const binarySha256 = crypto.createHash("sha256").update(fs.readFileSync(binary)).digest("hex");
const child = spawn(binary, argv, { stdio: ["pipe", "pipe", "pipe"] });
const identity = () => ({ pid: child.pid });

// Parsing is a side channel: forwarding never waits for a complete or valid frame.
function observer(direction) {
  let pending = Buffer.alloc(0);
  let parsing = true;
  return (chunk) => {
    record({ kind: "chunk", direction, ...identity(), chunkBase64: chunk.toString("base64") });
    if (!parsing) return;
    pending = Buffer.concat([pending, chunk]);
    for (;;) {
      const headerEnd = pending.indexOf("\r\n\r\n");
      if (headerEnd < 0) {
        if (pending.length > 65536) parsing = false;
        return;
      }
      const length = /^Content-Length:\s*(\d+)\s*$/im.exec(
        pending.subarray(0, headerEnd).toString("latin1"),
      );
      if (!length || !Number.isSafeInteger(Number(length[1]))) {
        parsing = false;
        return;
      }
      const end = headerEnd + 4 + Number(length[1]);
      if (pending.length < end) return;
      const frame = pending.subarray(0, end);
      pending = pending.subarray(end);
      try {
        const packet = JSON.parse(frame.subarray(headerEnd + 4).toString("utf8"));
        record({
          kind: "frame",
          direction,
          ...identity(),
          packet,
          frameBase64: frame.toString("base64"),
        });
      } catch {
        // Non-protocol output is retained in full by the raw chunk records.
      }
    }
  };
}

const observeClient = observer("client");
process.stdin.on("data", observeClient);
process.stdin.pipe(child.stdin);
child.stdout.on("data", observer("server"));
child.stdout.pipe(process.stdout, { end: false });
child.stderr.on("data", (chunk) => {
  record({ kind: "stderr", ...identity(), chunkBase64: chunk.toString("base64") });
});
child.stderr.pipe(process.stderr, { end: false });
child.on("spawn", () => record({ kind: "spawn", ...identity(), argv, binary, binarySha256 }));
child.on("error", (error) => {
  record({ kind: "error", ...identity(), message: error.message, code: error.code });
});
child.stdin.on("error", (error) => {
  record({ kind: "stdin-error", ...identity(), message: error.message, code: error.code });
});
const signals = ["SIGINT", "SIGTERM", "SIGHUP"];
const handlers = new Map(signals.map((signal) => [signal, () => child.kill(signal)]));
for (const [signal, handler] of handlers) process.on(signal, handler);
child.on("close", (code, signal) => {
  process.stdin.unpipe(child.stdin);
  process.stdin.removeListener("data", observeClient);
  process.stdin.pause();
  child.stdin.destroy();
  for (const [name, handler] of handlers) process.removeListener(name, handler);
  record({ kind: "exit", ...identity(), code, signal });
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code !== null && code >= 0 ? code : 1;
});
