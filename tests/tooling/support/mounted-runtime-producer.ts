// Only this producer loads the bundler and its native addons. It must exit
// naturally before an observer may import its generated JavaScript.
import assert from "node:assert/strict";
import { build } from "vite-plus";
import { vueVaporRuntimeEntry } from "./vue-vapor-release.mjs";

const inputChunks: Buffer[] = [];
for await (const chunk of process.stdin) inputChunks.push(chunk);
const input = JSON.parse(Buffer.concat(inputChunks).toString("utf8"));
assert.deepEqual(Object.keys(input), ["production"]);
const { production } = input;
assert.equal(typeof production, "boolean", "production runtime mode must be explicit");

const result = await build({
  configFile: false,
  logLevel: "silent",
  define: {
    "process.env.NODE_ENV": JSON.stringify(production ? "production" : "development"),
    __VUE_OPTIONS_API__: "true",
    __VUE_PROD_DEVTOOLS__: "false",
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: production ? "false" : "true",
  },
  build: {
    write: false,
    minify: production,
    lib: {
      entry: vueVaporRuntimeEntry,
      formats: ["es"],
    },
  },
});
const outputs = Array.isArray(result) ? result : [result];
const chunks = outputs.flatMap((output) => {
  assert.ok("output" in output, "runtime producer must return a completed build");
  return output.output.filter((item) => item.type === "chunk");
});
assert.equal(chunks.length, 1, "Vue runtime must bundle into one self-contained module");
await new Promise<void>((resolve, reject) => {
  process.stdout.write(`${JSON.stringify({ code: chunks[0].code })}\n`, (error) => {
    if (error) reject(error);
    else resolve();
  });
});
