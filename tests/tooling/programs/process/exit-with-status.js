const [status, delay] = process.argv.slice(2);
if (delay) setTimeout(() => process.exit(Number(status)), Number(delay));
else process.exitCode = Number(status);
