const supportModule = process.argv[2];

const { runtimeProcessEvidence, withMountedRuntimeDevtools } = await import(supportModule);
const evidence = runtimeProcessEvidence();
evidence.phase("trace", { backend: "vdom", fixtureIndex: 0 });
await withMountedRuntimeDevtools(
  false,
  () => {
    evidence.phase("close mounted DOM");
    return new Promise(() => {});
  },
  async () => {
    throw new Error("original trace rejection witness");
  },
  { enabled: false, onFailure: evidence.failure },
);
