process.stdin.once("data", (chunk) => process.stdout.write("ACK:" + chunk));
process.stdin.resume();
