import fs from "node:fs";
import { mount } from "@vue/test-utils";
import { expect, test } from "vitest";
import Hello from "./Hello.vue";

test("renders the actual compiled Vue component", () => {
  const wrapper = mount(Hello, { props: { name: "vize" } });
  const text = wrapper.text();
  const receipt = process.env.VIZE_CACHE_TEST_RECEIPT;
  if (!receipt) throw new Error("VIZE_CACHE_TEST_RECEIPT is required");
  fs.appendFileSync(receipt, JSON.stringify({ pid: process.pid, text }) + "\n");
  wrapper.unmount();
  expect(text).toBe(process.env.VIZE_CACHE_EXPECTED_TEXT);
});
