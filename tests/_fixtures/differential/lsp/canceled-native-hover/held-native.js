#!/usr/bin/env node
// Test-only byte-preserving gate around the actual configured native process.
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");
const gate = JSON.parse(fs.readFileSync(path.join(__dirname, "gate.json"), "utf8"));
const args = process.argv.slice(2);
const framed = args.includes("--lsp");
const child = spawn(gate.runtime, args, {
  stdio: framed ? ["pipe", "pipe", "inherit"] : "inherit",
});
fs.appendFileSync(path.join(gate.root, "native.pids"), `${child.pid}\n`);
child.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on("exit", (code, signal) => {
  fs.appendFileSync(path.join(gate.root, "native.exits"), JSON.stringify({ code, signal }) + "\n");
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code;
});
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.on(signal, () => {
    child.kill(signal);
    process.exitCode = 1;
  });
}

if (framed) {
  let selected;
  const parse = (consume) => {
    let buffered = Buffer.alloc(0);
    return (chunk) => {
      buffered = Buffer.concat([buffered, chunk]);
      while (true) {
        const boundary = buffered.indexOf("\r\n\r\n");
        if (boundary < 0) return;
        const match = /^Content-Length: (\d+)$/im.exec(buffered.subarray(0, boundary).toString());
        if (!match) throw new Error("native frame has no Content-Length");
        const end = boundary + 4 + Number(match[1]);
        if (buffered.length < end) return;
        const frame = buffered.subarray(0, end);
        const body = JSON.parse(buffered.subarray(boundary + 4, end));
        buffered = buffered.subarray(end);
        consume(frame, body);
      }
    };
  };
  process.stdin.on(
    "data",
    parse((frame, message) => {
      if (message.method === "textDocument/hover" && fs.existsSync(path.join(gate.root, "arm"))) {
        try {
          fs.writeFileSync(path.join(gate.root, "claimed"), "", { flag: "wx" });
          selected = message.id;
          fs.writeFileSync(path.join(gate.root, "held-request.bin"), frame);
          fs.writeFileSync(path.join(gate.root, "held-request.json"), JSON.stringify(message));
        } catch (error) {
          if (error.code !== "EEXIST") throw error;
        }
      }
      child.stdin.write(frame);
    }),
  );
  process.stdin.on("end", () => child.stdin.end());
  let output = Promise.resolve();
  child.stdout.on(
    "data",
    parse((frame, message) => {
      output = output
        .then(async () => {
          if (selected !== undefined && message.id === selected) {
            fs.writeFileSync(path.join(gate.root, "held-response.bin"), frame);
            fs.writeFileSync(path.join(gate.root, "held-response.json"), JSON.stringify(message));
            fs.writeFileSync(path.join(gate.root, "entered"), "");
            const until = Date.now() + 10000;
            while (!fs.existsSync(path.join(gate.root, "release"))) {
              if (Date.now() >= until)
                throw new Error("test native response gate was never released");
              await new Promise((resolve) => setTimeout(resolve, 5));
            }
            selected = undefined;
          }
          process.stdout.write(frame);
        })
        .catch((error) => {
          console.error(error);
          child.kill();
          process.exitCode = 1;
        });
    }),
  );
}
