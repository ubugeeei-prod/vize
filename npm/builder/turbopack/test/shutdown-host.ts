const [behavior, code] = process.argv.slice(2);
const status = Number(code);
if (behavior === "exit") process.exit(status);
if (behavior === "handle") process.on("SIGTERM", () => process.exit(status));
process.stdout.write("ready");
setInterval(() => {}, 1_000);
