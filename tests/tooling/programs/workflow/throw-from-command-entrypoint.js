const commandRuntime = process.argv[2];

const { runEntrypoint } = await import(commandRuntime);
runEntrypoint(() => {
  throw new Error("boom");
});
