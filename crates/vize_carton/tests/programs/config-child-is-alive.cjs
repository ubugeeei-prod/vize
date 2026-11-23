try {
  process.kill(Number(process.argv[2]), 0);
  process.exitCode = 1;
} catch (error) {
  process.exitCode = error.code === "ESRCH" ? 0 : 2;
}
