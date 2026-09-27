import { expect, it } from "vite-plus/test";
import { loadWasm, isUsingMock } from "../src/wasm/index";

it("real WASM formatter selects Vue 2 filters and retains default Vue 3 output", async () => {
  const wasm = await loadWasm();
  expect(isUsingMock()).toBe(false);
  const template = "<p>{{ message | format-date('en') }}</p>";
  const source = `<template>${template}</template>\n`;
  for (const vueVersion of [undefined, "3", "2", "2.7"]) {
    const options = vueVersion === undefined ? {} : { vueVersion };
    const expression =
      vueVersion === "2" || vueVersion === "2.7"
        ? 'message | format-date("en")'
        : 'message | (format - date("en"))';
    const expectedTemplate = `<p>{{ ${expression} }}</p>`;
    const expectedSfc = `<template>\n  ${expectedTemplate}\n</template>\n`;
    expect(wasm.formatTemplate(template, options)).toEqual({
      code: expectedTemplate,
      changed: true,
    });
    expect(wasm.formatSfc(source, options)).toEqual({ code: expectedSfc, changed: true });
    for (let pass = 2; pass <= 3; pass += 1) {
      expect(wasm.formatTemplate(expectedTemplate, options)).toEqual({
        code: expectedTemplate,
        changed: false,
      });
      expect(wasm.formatSfc(expectedSfc, options)).toEqual({ code: expectedSfc, changed: false });
    }
  }
  for (const vueVersion of ["unknown", "0", "2.0"]) {
    expect(() => wasm.formatTemplate(template, { vueVersion })).toThrow();
    expect(() => wasm.formatSfc(source, { vueVersion })).toThrow();
  }
});
