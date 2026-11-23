const loaderModule = process.argv[2];

const { importConfigModule } = await import(loaderModule);
const loaded = await importConfigModule(process.argv[3]);
process.stdout.write(JSON.stringify(loaded.default));
