#!/usr/bin/env node
// Test-only byte-preserving gate around the actual configured native process.
const fs = require("node:fs");
const { createHash } = require("node:crypto");
const path = require("node:path");
const { spawn } = require("node:child_process");
const gate = JSON.parse(fs.readFileSync(path.join(__dirname, "gate.json"), "utf8"));
const args = process.argv.slice(2);
const framed = args.includes("--lsp");
const witnessed = gate.physicalWitness === true;
const piped = framed || witnessed;
const child = spawn(gate.runtime, args, {
  stdio: piped ? ["pipe", "pipe", "inherit"] : "inherit",
});
if (witnessed) {
  const identity = (pid) => {
    const wholeStatBefore = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
    const executable = fs.realpathSync(`/proc/${pid}/exe`);
    const executableSha256 = createHash("sha256")
      .update(fs.readFileSync(`/proc/${pid}/exe`))
      .digest("hex");
    const command = fs.readFileSync(`/proc/${pid}/cmdline`);
    const wholeStatAfter = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
    const fields = (stat) =>
      stat
        .slice(stat.lastIndexOf(") ") + 2)
        .trim()
        .split(/\s+/);
    const before = fields(wholeStatBefore);
    const after = fields(wholeStatAfter);
    if (
      !/^\d+$/.test(before[19]) ||
      before[19] !== after[19] ||
      before[1] !== after[1] ||
      before[0] === "Z" ||
      after[0] === "Z"
    )
      throw new Error("physical process identity changed before protocol forwarding");
    return {
      pid,
      birth_ticks: Number(before[19]),
      parent: Number(before[1]),
      executable,
      executableSha256,
      commandBase64: command.toString("base64"),
      commandArguments: command.toString("utf8").split("\0").filter(Boolean),
      wholeStatBefore,
      wholeStatAfter,
    };
  };
  const invocation =
    args.includes("--lsp") && args.includes("--stdio") && !args.includes("--api")
      ? "editor-lsp"
      : args.includes("--api") && !args.includes("--lsp")
        ? "project-api"
        : args.length === 1 && args[0] === "--version"
          ? "version-probe"
          : "unknown";
  if (invocation === "unknown")
    throw new Error(`unclassified native invocation: ${JSON.stringify(args)}`);
  const proxy = identity(process.pid);
  const native = identity(child.pid);
  if (native.parent !== proxy.pid)
    throw new Error("native process is not this proxy's direct child");
  fs.appendFileSync(
    path.join(gate.root, "physical-starts.jsonl"),
    JSON.stringify({ phase: "before-protocol-forwarding", invocation, args, proxy, native }) + "\n",
  );
}
fs.appendFileSync(path.join(gate.root, "native.pids"), `${child.pid}\n`);
child.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on("exit", (code, signal) => {
  fs.appendFileSync(path.join(gate.root, "native.exits"), JSON.stringify({ code, signal }) + "\n");
  if (witnessed) {
    fs.appendFileSync(
      path.join(gate.root, "physical-terminals.jsonl"),
      JSON.stringify({ proxyPid: process.pid, nativePid: child.pid, code, signal }) + "\n",
    );
    if (!framed) {
      process.stdin.unpipe(child.stdin);
      process.stdin.pause();
      child.stdin.destroy();
    }
  }
  if (signal) process.kill(process.pid, signal);
  else process.exitCode = code;
});
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.on(signal, () => {
    child.kill(signal);
    process.exitCode = 1;
  });
}

if (witnessed && !framed) {
  // API/probe bytes remain opaque; connect only after the complete birth witness.
  process.stdin.pipe(child.stdin);
  child.stdout.pipe(process.stdout);
}

if (framed) {
  let selected;
  let shutdownId;
  let shutdownAcknowledged = false;
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
      if (message.method === "shutdown") shutdownId = message.id;
      if (message.method === "exit" && shutdownAcknowledged) {
        // This foreign proxy receives standard LSP Exit. Its actual pinned
        // native child needs EOF after the acknowledged Shutdown response.
        fs.writeFileSync(path.join(gate.root, "translated-exit.bin"), frame);
        child.stdin.end();
        return;
      }
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
  process.stdin.on("end", () => {
    if (!child.stdin.writableEnded) child.stdin.end();
  });
  let output = Promise.resolve();
  child.stdout.on(
    "data",
    parse((frame, message) => {
      if (shutdownId !== undefined && message.id === shutdownId && message.result === null) {
        shutdownAcknowledged = true;
        fs.writeFileSync(path.join(gate.root, "shutdown-response.bin"), frame);
      }
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
