const observerModule = process.argv[2];

const fault = await import(observerModule);
await fault.main(["--", "/nonexistent-vize-pure-fixture-command"], "linux");
