const loaded = await import("@vizejs/nuxt");
process.stdout.write(
  JSON.stringify({ type: typeof loaded.default, viteLoads: globalThis[process.argv[2]] }),
);
