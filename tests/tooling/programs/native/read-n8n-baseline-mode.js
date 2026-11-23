const selectorModule = process.argv[2];

const { baselineMode } = await import(selectorModule);
process.stdout.write(baselineMode());
