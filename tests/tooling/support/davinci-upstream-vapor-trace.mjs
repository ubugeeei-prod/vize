import assert from "node:assert/strict";
import { createRequire } from "node:module";
import process from "node:process";
import { traceMountedBackend } from "./davinci-mounted-trace.mjs";

const require = createRequire(import.meta.url);
// The pinned beta SFC compiler depends on this exact compiler-vapor version.
// Resolve through it so the official compiler and the mounted Vue runtime
// cannot silently drift to different releases.
const fromSfc = createRequire(require.resolve("@vue/compiler-sfc"));
const compiler = fromSfc("@vue/compiler-vapor");
const compilerVersion = fromSfc("@vue/compiler-vapor/package.json").version;
assert.equal(compilerVersion, require("vue/package.json").version);

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const { source, context, steps } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(typeof source, "string");
assert.ok(source.length > 0);
assert.ok(context && typeof context === "object" && !Array.isArray(context));
assert.ok(Array.isArray(steps) && steps.length > 0);

const compiled = compiler.compile(source, { mode: "module", prefixIdentifiers: true });
assert.ok(typeof compiled.code === "string" && compiled.code.length > 0);
process.stdout.write(
  `${JSON.stringify(
    await traceMountedBackend({
      backend: "vapor",
      code: compiled.code,
      context,
      steps,
      identities: true,
    }),
  )}\n`,
);
