// Independent locked Vue VDOM oracle for the authored #7886 update controls.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { vueVaporVersion } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const input = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(input.backend, "vapor");
assert.equal(vueVaporVersion, "3.6.0-rc.9");
const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue-vapor-runtime/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
assert.equal(compiler.version, vueVaporVersion);

function stock(source, filename) {
  const parsed = compiler.parse(source, { filename });
  assert.deepEqual(parsed.errors, []);
  const descriptor = parsed.descriptor;
  const script = compiler.compileScript(descriptor, {
    id: "slot-defaults",
    genDefaultAs: "__component",
  });
  const template = compiler.compileTemplate({
    source: descriptor.template.content,
    filename,
    id: "slot-defaults",
    compilerOptions: { bindingMetadata: script.bindings },
  });
  assert.deepEqual(template.errors, []);
  assert.deepEqual(template.tips, []);
  return `${script.content}\n${template.code}\n__component.render = render;\nexport default __component;\n`;
}

function trace(value) {
  const output = spawnSync(
    process.execPath,
    [fileURLToPath(new URL("vapor-sfc-runtime.mjs", import.meta.url))],
    { input: JSON.stringify(value), encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
  );
  assert.ifError(output.error);
  assert.equal(
    output.status,
    0,
    `status=${output.status}\nstdout=${output.stdout}\nstderr=${output.stderr}\n${value.code}\n${value.child}`,
  );
  return JSON.parse(output.stdout);
}

const official = {
  ...input,
  backend: "vdom",
  code: stock(input.appSource, "App.vue"),
  child: stock(input.childSource, "Child.vue"),
};
process.stdout.write(JSON.stringify({ current: trace(input), oracle: trace(official) }));
