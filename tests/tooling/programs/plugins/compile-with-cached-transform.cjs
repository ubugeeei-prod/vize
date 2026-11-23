const fixture = JSON.parse(process.argv[2]);

const native = require(fixture.nativeLibrary);
const output = native.compileWithTransformPlugins(
  fixture.source,
  [
    {
      name: "cold-transform",
      version: "1",
      fingerprint: "design-system-static-class-v1",
      cacheInputs: [],
      run() {
        throw new Error("persistent hit must not call JS");
      },
    },
  ],
  { cache: true, cacheDir: fixture.cacheDirectory },
);
process.stdout.write(JSON.stringify(output));
