import { spawn } from "node:child_process";
import fs from "node:fs";

export function startTsserver(entry, cwd, output) {
  const packets = [];
  const pending = new Map();
  let sequence = 0,
    buffer = Buffer.alloc(0),
    bytes = 0;
  const child = spawn(
    process.execPath,
    [
      entry,
      "--disableAutomaticTypingAcquisition",
      "--logVerbosity",
      "verbose",
      "--logFile",
      `${output}/tsserver.log`,
    ],
    { cwd, stdio: ["pipe", "pipe", "pipe"] },
  );
  let stderr = "";
  let exited = false;
  // A killed child must not replace the original failure with EPIPE.
  child.stdin.on("error", () => {});
  const timeout = setTimeout(() => child.kill("SIGKILL"), 90_000);
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
    if (stderr.length > 1_000_000) child.kill("SIGKILL");
  });
  child.stdout.on("data", (chunk) => {
    bytes += chunk.length;
    if (bytes > 16_000_000) {
      child.kill("SIGKILL");
      return;
    }
    buffer = Buffer.concat([buffer, chunk]);
    while (true) {
      const index = buffer.indexOf("\r\n\r\n");
      if (index < 0) break;
      const length = Number(
        /Content-Length: (\d+)/i.exec(buffer.subarray(0, index).toString())?.[1],
      );
      if (!Number.isSafeInteger(length)) throw new Error("Invalid tsserver frame");
      if (buffer.length < index + 4 + length) break;
      const packet = JSON.parse(buffer.subarray(index + 4, index + 4 + length));
      buffer = buffer.subarray(index + 4 + length);
      packets.push({ direction: "response", packet });
      if (packet.type === "response") {
        const request = pending.get(packet.request_seq);
        if (request) {
          pending.delete(packet.request_seq);
          clearTimeout(request.timeout);
          request.resolve(packet);
        }
      }
    }
  });
  const completion = new Promise((resolve) =>
    child.once("exit", (code, signal) => {
      exited = true;
      clearTimeout(timeout);
      for (const request of pending.values()) {
        clearTimeout(request.timeout);
        request.reject(new Error(`tsserver exited ${code}/${signal}`));
      }
      pending.clear();
      resolve({ code, signal });
    }),
  );
  function send(command, args, response = true) {
    const packet = { seq: ++sequence, type: "request", command, arguments: args };
    packets.push({ direction: "request", packet });
    const result = response
      ? new Promise((resolve, reject) => {
          const timer = setTimeout(() => {
            pending.delete(packet.seq);
            reject(new Error(`No response for ${command}`));
            child.kill("SIGKILL");
          }, 20_000);
          pending.set(packet.seq, { resolve, reject, timeout: timer });
        })
      : Promise.resolve();
    if (!exited) child.stdin.write(`${JSON.stringify(packet)}\n`);
    return result;
  }
  let stopped;
  return {
    send,
    stop() {
      if (stopped) return stopped;
      stopped = (async () => {
        try {
          await send("exit", {}, false);
          child.stdin.end();
          await completion;
        } finally {
          child.kill("SIGKILL");
          await completion;
          fs.writeFileSync(`${output}/packets.json`, JSON.stringify(packets, null, 2));
          fs.writeFileSync(`${output}/stderr.log`, stderr);
        }
      })();
      return stopped;
    },
  };
}
export function position(text, needle, offset = 1) {
  const index = text.indexOf(needle);
  if (index < 0) throw new Error(`Missing marker ${needle}`);
  const lines = text.slice(0, index + offset).split("\n");
  return { line: lines.length, offset: lines.at(-1).length + 1 };
}
