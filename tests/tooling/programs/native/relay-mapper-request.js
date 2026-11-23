let body = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => {
  body += chunk;
});
process.stdin.on("end", () => {
  process.stdout.write("OUT:" + body);
  process.stderr.write("ERR");
  process.exitCode = 23;
});
