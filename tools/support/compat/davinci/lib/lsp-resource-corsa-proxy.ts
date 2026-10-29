/** Byte-preserving Corsa proxy installed by the resource measurement gate. */
export const resourceCorsaProxySource = String.raw`
const fs = require("node:fs");
const path = require("node:path");
const { spawn } = require("node:child_process");
const gate = JSON.parse(fs.readFileSync(path.join(__dirname, "gate.json"), "utf8"));
const args = process.argv.slice(2);
const framed = args.includes("--lsp");
const mirroredDocuments = new Map();

function procIdentity(role, pid) {
  const stat = fs.readFileSync("/proc/" + pid + "/stat", "utf8");
  const fields = stat.slice(stat.lastIndexOf(") ") + 2).trim().split(/\s+/);
  return {
    role,
    transport: framed ? "lsp" : "other",
    pid,
    birth_ticks: Number(fields[19]),
    executable: fs.realpathSync("/proc/" + pid + "/exe"),
  };
}

function recordProcess(identity) {
  fs.appendFileSync(path.join(gate.root, "processes.jsonl"), JSON.stringify(identity) + "\n");
}

recordProcess(procIdentity("proxy", process.pid));
const child = spawn(gate.runtime, args, { stdio: framed ? ["pipe", "pipe", "inherit"] : "inherit" });
child.once("spawn", () => recordProcess(procIdentity("native", child.pid)));
child.on("error", (error) => {
  console.error(error);
  process.exitCode = 1;
});
child.on("exit", (code, signal) => {
  fs.appendFileSync(path.join(gate.root, "native-exits.jsonl"), JSON.stringify({ code, signal }) + "\n");
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  }
  else process.exitCode = code;
});
for (const signal of ["SIGTERM", "SIGINT"]) {
  process.on(signal, () => {
    child.kill(signal);
    process.exitCode = 1;
  });
}

function parseFrames(consume) {
  let buffered = Buffer.alloc(0);
  return (chunk) => {
    buffered = Buffer.concat([buffered, chunk]);
    while (true) {
      const boundary = buffered.indexOf("\r\n\r\n");
      if (boundary < 0) return;
      const match = /^Content-Length:\s*(\d+)$/im.exec(buffered.subarray(0, boundary).toString());
      if (!match) throw new Error("native frame has no Content-Length");
      const end = boundary + 4 + Number(match[1]);
      if (buffered.length < end) return;
      const frame = buffered.subarray(0, end);
      const message = JSON.parse(buffered.subarray(boundary + 4, end).toString("utf8"));
      buffered = buffered.subarray(end);
      consume(frame, message);
    }
  };
}

function tokenDirectory(token) {
  return path.join(gate.root, "tokens", String(token).padStart(2, "0"));
}

function armedToken() {
  try {
    const value = JSON.parse(fs.readFileSync(path.join(gate.root, "armed.json"), "utf8"));
    if (!Number.isSafeInteger(value.token) || value.token <= 0) throw new Error("invalid gate token");
    return value.token;
  } catch (error) {
    if (error && error.code === "ENOENT") return undefined;
    throw error;
  }
}

function lineStartOffset(text, line) {
  if (!Number.isSafeInteger(line) || line < 0) throw new Error("invalid LSP line");
  let current = 0;
  let offset = 0;
  while (current < line) {
    const lf = text.indexOf("\n", offset);
    const cr = text.indexOf("\r", offset);
    const boundary = lf < 0 ? cr : cr < 0 ? lf : Math.min(lf, cr);
    if (boundary < 0) throw new Error("LSP line is outside mirrored document");
    offset = boundary + (text[boundary] === "\r" && text[boundary + 1] === "\n" ? 2 : 1);
    current++;
  }
  return offset;
}

function positionOffset(text, position) {
  if (!position || !Number.isSafeInteger(position.character) || position.character < 0) {
    throw new Error("invalid LSP character");
  }
  const start = lineStartOffset(text, position.line);
  const lf = text.indexOf("\n", start);
  const cr = text.indexOf("\r", start);
  const end = lf < 0 ? (cr < 0 ? text.length : cr) : cr < 0 ? lf : Math.min(lf, cr);
  if (start + position.character > end) throw new Error("LSP character is outside mirrored line");
  return start + position.character;
}

function updateMirroredDocument(message) {
  const params = message.params;
  if (message.method === "textDocument/didOpen") {
    const document = params && params.textDocument;
    if (!document || typeof document.uri !== "string" || typeof document.text !== "string") {
      throw new Error("invalid mirrored didOpen");
    }
    mirroredDocuments.set(document.uri, {
      openedUri: document.uri,
      text: document.text,
      version: document.version ?? null,
    });
    return;
  }
  if (message.method === "textDocument/didChange") {
    const document = params && params.textDocument;
    const changes = params && params.contentChanges;
    const previous = document && mirroredDocuments.get(document.uri);
    if (!document || !previous || !Array.isArray(changes)) {
      throw new Error("invalid mirrored didChange");
    }
    let text = previous.text;
    for (const change of changes) {
      if (!change || typeof change.text !== "string") throw new Error("invalid mirrored change");
      if (!change.range) {
        text = change.text;
        continue;
      }
      const start = positionOffset(text, change.range.start);
      const end = positionOffset(text, change.range.end);
      if (end < start) throw new Error("reversed mirrored change range");
      text = text.slice(0, start) + change.text + text.slice(end);
    }
    mirroredDocuments.set(document.uri, {
      openedUri: previous.openedUri,
      text,
      version: document.version ?? previous.version,
    });
    return;
  }
  if (message.method === "textDocument/didClose") {
    const uri = params && params.textDocument && params.textDocument.uri;
    if (typeof uri === "string") mirroredDocuments.delete(uri);
  }
}

function writeFrame(frame) {
  return new Promise((resolve, reject) => {
    process.stdout.write(frame, (error) => error ? reject(error) : resolve());
  });
}

async function awaitRelease(directory) {
  const deadline = Date.now() + 120000;
  while (!fs.existsSync(path.join(directory, "release"))) {
    if (Date.now() >= deadline) throw new Error("resource native response gate was never released");
    await new Promise((resolve) => setTimeout(resolve, 2));
  }
}

if (framed) {
  let selected;
  let lastToken = 0;
  process.stdin.on("data", parseFrames((frame, message) => {
    updateMirroredDocument(message);
    if (message.method === "textDocument/hover" && selected === undefined) {
      const token = armedToken();
      if (token !== undefined && token > lastToken) {
        const directory = tokenDirectory(token);
        try {
          fs.writeFileSync(path.join(directory, "claimed"), "", { flag: "wx" });
          const uri = message.params && message.params.textDocument && message.params.textDocument.uri;
          const document = typeof uri === "string" && mirroredDocuments.get(uri);
          if (!document) throw new Error("native hover has no exact mirrored source clone");
          selected = { token, id: message.id, directory };
          fs.writeFileSync(path.join(directory, "request.bin"), frame);
          fs.writeFileSync(path.join(directory, "request.json"), JSON.stringify(message));
          fs.writeFileSync(path.join(directory, "canonical-source.bin"), document.text);
          fs.writeFileSync(
            path.join(directory, "canonical.json"),
            JSON.stringify({ opened_uri: document.openedUri, version: document.version }),
          );
        } catch (error) {
          if (!error || error.code !== "EEXIST") throw error;
        }
      }
    }
    child.stdin.write(frame);
  }));
  process.stdin.on("end", () => child.stdin.end());
  let output = Promise.resolve();
  child.stdout.on("data", parseFrames((frame, message) => {
    output = output.then(async () => {
      if (selected !== undefined && message.id === selected.id) {
        const held = selected;
        fs.writeFileSync(path.join(held.directory, "response.bin"), frame);
        fs.writeFileSync(path.join(held.directory, "response.json"), JSON.stringify(message));
        fs.writeFileSync(
          path.join(held.directory, "entered.json"),
          JSON.stringify({ token: held.token, entered_ns: process.hrtime.bigint().toString() }),
        );
        await awaitRelease(held.directory);
        await writeFrame(frame);
        fs.writeFileSync(
          path.join(held.directory, "released.json"),
          JSON.stringify({ token: held.token, released_ns: process.hrtime.bigint().toString() }),
        );
        lastToken = held.token;
        selected = undefined;
        return;
      }
      await writeFrame(frame);
    }).catch((error) => {
      console.error(error);
      child.kill("SIGTERM");
      process.exitCode = 1;
    });
  }));
}
`;
