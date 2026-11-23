#!/usr/bin/env node
let buffer = Buffer.alloc(0);

function write(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`);
  process.stdout.write(body);
}

function handle(message) {
  const method = message.method;
  if ((method === "shutdown" || method === "exit") && "params" in message) {
    process.stderr.write(
      `${method} unexpectedly included params: ${JSON.stringify(message.params)}\n`,
    );
    process.exit(1);
  }
  if (method === "initialize") {
    write({ jsonrpc: "2.0", id: message.id, result: { capabilities: {} } });
  } else if (method === "shutdown") {
    write({ jsonrpc: "2.0", id: message.id, result: null });
  } else if (method === "exit") {
    process.exit(0);
  }
}

process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const end = buffer.indexOf("\r\n\r\n");
    if (end < 0) return;
    const length = Number(
      /content-length:\s*(\d+)/i.exec(buffer.subarray(0, end).toString("ascii"))[1],
    );
    if (buffer.length < end + 4 + length) return;
    const body = buffer.subarray(end + 4, end + 4 + length).toString("utf8");
    buffer = buffer.subarray(end + 4 + length);
    handle(JSON.parse(body));
  }
});
process.stdin.on("end", () => {
  process.stderr.write("transport closed before exit\n");
  process.exit(1);
});
