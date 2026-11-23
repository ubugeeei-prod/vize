const fixture = JSON.parse(process.argv[2]);

const native = require(fixture.nativeLibrary);
let calls = 0;
const plugin = {
  name: "persistent-output",
  family: "output",
  version: "1",
  fingerprint: "license-v1",
  cacheInputs: [],
  run() {
    if (!fixture.mayCall) throw new Error("cache hit called JavaScript");
    calls++;
    return JSON.stringify([{ placement: "prepend", comment: "persistent license" }]);
  },
};
const result = native.compileWithOutputPlugins(fixture.source, [plugin], fixture.options, {
  cache: true,
  cacheDir: fixture.cacheDirectory,
});
process.stdout.write(JSON.stringify({ result: result.result, cost: result.plugins[0], calls }));
