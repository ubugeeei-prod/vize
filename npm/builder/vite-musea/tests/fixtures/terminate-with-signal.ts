const signal: NodeJS.Signals = "SIGTERM";
process.kill(process.pid, signal);
