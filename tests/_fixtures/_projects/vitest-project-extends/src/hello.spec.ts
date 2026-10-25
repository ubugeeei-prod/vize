import { test, expect } from "vitest";
import { createSSRApp } from "vue";
import { renderToString } from "vue/server-renderer";
import Hello from "./Hello.vue";
test("component module loads and renders", async () => {
  expect(Hello).toBeTruthy();
  expect(await renderToString(createSSRApp(Hello, { name: "World" }))).toBe("<p>Hello World</p>");
});
