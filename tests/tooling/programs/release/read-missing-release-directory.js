const runtimeModule = process.argv[2];

const { readDir } = await import(runtimeModule);
console.log(JSON.stringify(readDir("missing")));
