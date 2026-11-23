let s = "";
process.stdin.on("data", (c) => (s += c)).on("end", () => process.stdout.write(JSON.parse(s)));
