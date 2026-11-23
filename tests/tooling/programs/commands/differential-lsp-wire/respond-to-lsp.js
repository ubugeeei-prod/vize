let buffer = Buffer.alloc(0);
process.stdin.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  while (true) {
    const end = buffer.indexOf("\r\n\r\n");
    if (end < 0) return;
    const length = Number(
      buffer
        .subarray(0, end)
        .toString()
        .match(/Content-Length: (\d+)/)[1],
    );
    if (buffer.length < end + 4 + length) return;
    const message = JSON.parse(buffer.subarray(end + 4, end + 4 + length));
    buffer = buffer.subarray(end + 4 + length);
    if (message.method === "exit") {
      process.exit(0);
    }
    if (message.id) {
      const body = JSON.stringify({
        jsonrpc: "2.0",
        id: message.id,
        result: message.method === "shutdown" ? null : "😀あ",
      });
      const frame = Buffer.from("Content-Length: " + Buffer.byteLength(body) + "\r\n\r\n" + body);
      process.stdout.write(frame.subarray(0, 7));
      process.stdout.write(frame.subarray(7));
    }
  }
});
