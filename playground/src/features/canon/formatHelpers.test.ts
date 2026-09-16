import { expect, it } from "vite-plus/test";
import { formatMessage } from "./formatHelpers";

it("highlights quoted types once without reprocessing generated HTML attributes", () => {
  expect(formatMessage(`Type 'HTMLInputElement' is not assignable to "HTMLButtonElement".`)).toBe(
    'Type <code class="msg-type">HTMLInputElement</code> is not assignable to <code class="msg-type">HTMLButtonElement</code>.',
  );
  expect(formatMessage(`Type '<script>&' is invalid.`)).toBe(
    'Type <code class="msg-type">&lt;script&gt;&amp;</code> is invalid.',
  );
});
