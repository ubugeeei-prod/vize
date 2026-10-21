import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);
const normalizationOptions = [
  "normalizeCssQuotes",
  "normalizeInlineStyles",
  "normalizeVForParens",
  "selfCloseVoidElements",
  "reindentPreprocessorStyles",
] as const;

test("shipped native formatter types accept the same line endings as generated config types", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-line-ending-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      'for (const endOfLine of ["lf", "crlf", "cr", "auto"] as const) {',
      "  const config: FormatterConfig = { endOfLine };",
      "  const native: FormatOptionsNapi = { endOfLine: config.endOfLine };",
      "  formatSfc('', native);",
      "}",
      'for (const preset of ["vize", "prettier", "compact", "documentation"] as const) {',
      "  formatSfc('', { preset, semi: false, sortAttributes: false });",
      "}",
      "// @ts-expect-error unknown presets must remain rejected.",
      "formatSfc('', { preset: 'docs' });",
      "// @ts-expect-error numeric presets must remain rejected.",
      "formatSfc('', { preset: 1 });",
      "// @ts-expect-error unknown modes must remain rejected.",
      "formatSfc('', { endOfLine: 'windows' });",
      "// @ts-expect-error numeric modes must remain rejected.",
      "formatSfc('', { endOfLine: 1 });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("config schema and shipped Node types share the Prettier normalizer booleans", (t) => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  for (const name of normalizationOptions) {
    assert.equal(schema.definitions.FormatterConfig.properties[name].type, "boolean");
  }

  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-normalizer-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      "const config: FormatterConfig = {",
      ...normalizationOptions.map((name) => `  ${name}: true,`),
      "};",
      "const native: FormatOptionsNapi = {",
      ...normalizationOptions.map((name) => `  ${name}: config.${name},`),
      "};",
      "formatSfc('', native);",
      ...normalizationOptions.flatMap((name) => [
        `// @ts-expect-error ${name} remains boolean-only.`,
        `formatSfc('', { ${name}: 'yes' });`,
      ]),
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("config schema and shipped Node types share script quote policies", (t) => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  assert.equal(schema.definitions.FormatterConfig.properties.jsxSingleQuote.type, "boolean");
  assert.deepEqual(schema.definitions.FormatterConfig.properties.quoteProps.enum, [
    "as-needed",
    "consistent",
    "preserve",
  ]);

  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-script-quote-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      'for (const quoteProps of ["as-needed", "consistent", "preserve"] as const) {',
      "  const config: FormatterConfig = { jsxSingleQuote: true, quoteProps };",
      "  const native: FormatOptionsNapi = {",
      "    jsxSingleQuote: config.jsxSingleQuote,",
      "    quoteProps: config.quoteProps,",
      "  };",
      "  formatSfc('', native);",
      "}",
      "// @ts-expect-error unknown property quote modes must remain rejected.",
      "formatSfc('', { quoteProps: 'keep' });",
      "// @ts-expect-error JSX quote preference remains boolean-only.",
      "formatSfc('', { jsxSingleQuote: 'yes' });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("config schema and shipped Node types share trailing comma modes", (t) => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  assert.deepEqual(schema.definitions.FormatterConfig.properties.trailingComma.enum, [
    "all",
    "none",
    "es5",
  ]);

  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-trailing-comma-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      'for (const trailingComma of ["all", "es5", "none"] as const) {',
      "  const config: FormatterConfig = { trailingComma };",
      "  const native: FormatOptionsNapi = { trailingComma: config.trailingComma };",
      "  formatSfc('', native);",
      "}",
      "// @ts-expect-error unknown trailing comma modes must remain rejected.",
      "formatSfc('', { trailingComma: 'never' });",
      "// @ts-expect-error trailing comma modes remain string-only.",
      "formatSfc('', { trailingComma: false });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("config schema and shipped Node types share arrow parentheses modes", (t) => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  assert.deepEqual(schema.definitions.FormatterConfig.properties.arrowParens.enum, [
    "always",
    "avoid",
  ]);

  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-arrow-parens-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      'for (const arrowParens of ["always", "avoid"] as const) {',
      "  const config: FormatterConfig = { arrowParens };",
      "  const native: FormatOptionsNapi = { arrowParens: config.arrowParens };",
      "  formatSfc('', native);",
      "}",
      "// @ts-expect-error unknown arrow parentheses modes must remain rejected.",
      "formatSfc('', { arrowParens: 'preserve' });",
      "// @ts-expect-error arrow parentheses modes remain string-only.",
      "formatSfc('', { arrowParens: false });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});

test("config schema and shipped Node types share bracket layout booleans", (t) => {
  const schema = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/schemas/vize.config.schema.json"), "utf8"),
  );
  assert.equal(schema.definitions.FormatterConfig.properties.bracketSpacing.type, "boolean");
  assert.equal(schema.definitions.FormatterConfig.properties.bracketSameLine.type, "boolean");

  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "glyph-bracket-layout-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { formatSfc, type FormatOptionsNapi } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { FormatterConfig } from ${JSON.stringify(path.join(root, "npm/cli/src/types/generated.js"))};`,
      "const config: FormatterConfig = { bracketSpacing: false, bracketSameLine: true };",
      "const native: FormatOptionsNapi = {",
      "  bracketSpacing: config.bracketSpacing,",
      "  bracketSameLine: config.bracketSameLine,",
      "};",
      "formatSfc('', native);",
      "// @ts-expect-error bracket spacing remains boolean-only.",
      "formatSfc('', { bracketSpacing: 'never' });",
      "// @ts-expect-error bracket placement remains boolean-only.",
      "formatSfc('', { bracketSameLine: 'always' });",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        types: [],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});
