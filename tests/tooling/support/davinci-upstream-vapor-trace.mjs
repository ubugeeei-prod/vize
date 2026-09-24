import assert from "node:assert/strict";
import process from "node:process";
import { traceMountedBackend } from "./davinci-mounted-trace.mjs";
import { officialCompilerVapor } from "./vue-vapor-release.mjs";

// The official compiler and the mounted runtime come from the same Vue
// release Vize's Vapor codegen targets (see `vue-vapor-release.mjs`).
const compiler = officialCompilerVapor;

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const {
  source,
  context,
  steps,
  components = {},
} = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(typeof source, "string");
assert.ok(source.length > 0);
assert.ok(context && typeof context === "object" && !Array.isArray(context));
assert.ok(Array.isArray(steps) && steps.length > 0);

const compiled = compiler.compile(source, { mode: "module", prefixIdentifiers: true });
assert.ok(typeof compiled.code === "string" && compiled.code.length > 0);
const compiledComponents = Object.fromEntries(
  Object.entries(components).map(([name, spec]) => [
    name,
    {
      ...spec,
      code: compiler.compile(spec.source, { mode: "module", prefixIdentifiers: true }).code,
    },
  ]),
);
process.stdout.write(
  `${JSON.stringify(
    await traceMountedBackend({
      backend: "vapor",
      code: compiled.code,
      context,
      steps,
      identities: true,
      components: compiledComponents,
    }),
  )}\n`,
);
