// Actual pinned Vue refs; full independently authored write-result vectors.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
const evidence = { vue: null, observations: [] };
try {
  assert.equal(process.env.NODE_ENV, "production");
  const fromUi = createRequire(new URL("../../../npm/ui/package.json", import.meta.url));
  const vue = fromUi("vue-vapor-runtime");
  assert.equal(vue.version, "3.6.0-rc.9");
  evidence.vue = vue.version;
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const rows = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  assert.equal(rows.length, 13);
  for (const row of rows) {
    for (const ref of [false, true]) {
      const context = {
        _isRef: vue.isRef,
        _unref: vue.unref,
        ref: vue.ref,
      };
      const run = compileFunction(
        `
        const { _isRef, _unref, ref } = context;
        let evt = asRef ? ref(1) : 1;
        let attr = asRef ? ref(3) : 3;
        const n = ref(2);
        let calls = 0;
        const _ctx = { next(value) { calls++; return value; } };
        const value = (${row.code});
        return { value, event: _unref(evt), attr: _unref(attr), calls, n: n.value };
      `,
        ["context", "asRef"],
      );
      const actual = run(context, ref);
      evidence.observations.push({ source: row.source, code: row.code, ref, actual });
      assert.deepEqual(actual, row.expected);
    }
  }
} catch (error) {
  evidence.error = { name: error.name, message: error.message, stack: error.stack };
  process.exitCode = 1;
} finally {
  await new Promise((resolve, reject) => {
    process.stdout.write(`${JSON.stringify(evidence)}\n`, (error) =>
      error ? reject(error) : resolve(),
    );
  });
}
