const supportModule = process.argv[2];

const { runtimeProcessEvidence } = await import(supportModule);
const evidence = runtimeProcessEvidence();
evidence.phase("close mounted DOM", { backend: "vdom", fixtureIndex: 0 });
await new Promise(() => {});
