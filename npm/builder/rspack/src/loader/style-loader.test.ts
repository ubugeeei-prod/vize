import assert from "node:assert/strict";
import { test } from "node:test";
import styleLoader from "./style-loader.ts";

void test("a removed style block emits empty CSS during HMR while invalid build requests remain errors", () => {
  for (const hot of [true, false]) {
    const errors: Error[] = [];
    let output: string | undefined;
    styleLoader.call(
      {
        hot,
        resourcePath: "/project/App.vue",
        resourceQuery: "?vue&type=style&index=1&lang=css",
        addDependency() {},
        emitError(error: Error) {
          errors.push(error);
        },
        async: () => (error: Error | null, code: string) => {
          assert.equal(error, null);
          output = code;
        },
      } as never,
      "<template><div /></template><style>div { color: red; }</style>",
    );
    assert.equal(output, "");
    assert.equal(errors.length, hot ? 0 : 1);
  }
});
