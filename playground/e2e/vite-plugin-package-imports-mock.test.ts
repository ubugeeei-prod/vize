import { expect, it, vi } from "vite-plus/test";
import { h } from "vue";
import { renderToString } from "vue/server-renderer";

import Parent from "./fixtures/package-imports-mock/parent.vue";

vi.mock("#mock-child/child.vue", () => ({
  default: { render: () => h("i", "stub") },
}));

it("mocks a Vue SFC imported through package.json imports", async () => {
  expect(await renderToString(h(Parent))).toContain("<i>stub</i>");
});
