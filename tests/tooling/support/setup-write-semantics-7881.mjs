// Actual pinned Vue refs; full independently authored write-result vectors.
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { Script } from "node:vm";
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
      let calls = 0;
      const scope = {
        _isRef: vue.isRef,
        _unref: vue.unref,
        evt: ref ? vue.ref(1) : 1,
        attr: ref ? vue.ref(3) : 3,
        n: vue.ref(2),
        _ctx: {
          next(value) {
            calls++;
            return value;
          },
        },
      };
      const value = new Script(`(${row.code})`).runInNewContext(scope);
      const actual = {
        value,
        event: vue.unref(scope.evt),
        attr: vue.unref(scope.attr),
        calls,
        n: scope.n.value,
      };
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
