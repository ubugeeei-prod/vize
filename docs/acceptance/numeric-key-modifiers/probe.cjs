const fs = require("node:fs");
const assert = require("node:assert/strict");
const path = require("node:path");
const { createRequire } = require("node:module");
// Pass the directory containing the exact pinned provider package.json.
const providerRequire = createRequire(path.resolve(process.argv[2], "package.json"));
const { ESLint } = providerRequire("eslint");
const parser = providerRequire("vue-eslint-parser");
const vue = providerRequire("eslint-plugin-vue");
const versions = Object.fromEntries(
  ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map((name) => [
    name,
    providerRequire(`${name}/package.json`).version,
  ]),
);
assert.deepEqual(versions, {
  eslint: "10.4.1",
  "eslint-plugin-vue": "10.9.2",
  "vue-eslint-parser": "10.4.1",
  "@typescript-eslint/parser": "8.65.0",
});
const modifiers = [
  "13.27",
  "27.13",
  "stop.13.27",
  "0.13",
  "9.13",
  "13.0",
  "13suffix",
  "9suffix.13",
  "f2.27",
  "+13",
  "-1",
  "-0.13",
  "0013",
  "0x13.27",
  "1e2.13",
  "999",
  "999.13",
  "13.999",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "0",
  "10",
  "12",
  "32",
  "65",
  "90",
  "112",
  "123",
  "144",
  "145",
];
modifiers.push(
  "enter",
  "f2",
  "stop",
  "",
  "13e2",
  "-13suffix",
  "+0.13",
  "0b13.27",
  "9".repeat(309) + ".13",
  "9".repeat(100) + ".13",
);
for (const code of [
  16,
  17,
  18,
  19,
  20,
  33,
  34,
  35,
  36,
  37,
  38,
  39,
  40,
  45,
  46,
  ...Array.from({ length: 26 }, (_, index) => 65 + index),
  ...Array.from({ length: 12 }, (_, index) => 112 + index),
]) {
  if (!modifiers.includes(String(code))) modifiers.push(String(code));
}
const eslint = new ESLint({
  overrideConfigFile: true,
  overrideConfig: [
    {
      files: ["**/*.vue"],
      languageOptions: { parser },
      plugins: { vue },
      rules: { "vue/no-deprecated-v-on-number-modifiers": "error" },
    },
  ],
});
(async () => {
  const records = [];
  for (const modifier of modifiers) {
    const source = `<template><input @keyup.${modifier}="submit" /></template>`;
    records.push({
      modifier,
      source,
      results: await eslint.lintText(source, { filePath: "Numeric.vue" }),
    });
  }
  fs.writeFileSync(
    path.join(__dirname, "provider.json"),
    JSON.stringify({ versions, records }, null, 2) + "\n",
  );
  console.log(`Captured ${records.length} complete ESLint results in provider.json`);
})();
