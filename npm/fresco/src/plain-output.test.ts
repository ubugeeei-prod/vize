import assert from "node:assert/strict";
import { test } from "vitest";
import { h } from "vue";

import { renderToString } from "./app.js";

test("renderToString keeps tabs and newlines and drops other control characters from text", () => {
  const output = renderToString(() =>
    h("box", { style: { flexDirection: "column" } }, [
      h("box", { internal_static: true }, [
        h("text", { internal_static_item: true, text: "log\u001b[2Jline" }),
      ]),
      h("text", { text: "bold\u001b[1m done\u0007" }),
      h("text", { text: "a\tb\nc\r\u0000\u007f\u009b31m\u0085d" }),
      h("input", { value: "typed\u001b[1A" }),
      h("raw", { lines: ["frame"] }),
    ]),
  );
  assert.equal(output, "log[2Jline\nbold[1m done\na\tb\nc31md\ntyped[1A");
});
