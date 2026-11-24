const fixture = JSON.parse(process.argv[2]);

const fault = await import(fixture.observerModule);
await fault.main(fixture.args, "linux");
