const fs = require("node:fs");
const chunks = [];
process.stdin.on("data", (chunk) => chunks.push(chunk));
process.stdin.on("end", () => {
  const input = JSON.parse(Buffer.concat(chunks).toString());
  for (const [fd, encoded] of [
    [1, input.stdout],
    [2, input.stderr],
  ]) {
    const bytes = Buffer.from(encoded, "hex");
    let at = 0;
    while (at < bytes.length) at += fs.writeSync(fd, bytes, at);
  }
  if (input.signal) process.kill(process.pid, input.signal);
  else process.exitCode = input.exit;
});
