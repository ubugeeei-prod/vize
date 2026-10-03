import assert from "node:assert/strict";
import fs from "node:fs";

const DEFAULT_OPTIONS: Record<string, unknown> = JSON.parse(
  fs.readFileSync(
    new URL("../_fixtures/differential/formatter/format-options-reference.json", import.meta.url),
    "utf8",
  ),
).formatOptions;

type Fixture = {
  api: string;
  vueVersion?: string;
  profile: string;
  importSorting?: { provided: boolean; setting: unknown; resolvedOptions: string };
  options: {
    base: string;
    internalOverrides: Record<string, boolean>;
    userOverrides?: Record<string, unknown>;
  };
};

export function configuredFormatterOptions(fixture: Fixture) {
  const internal = fixture.profile === "skip_script_stabilization";
  const overrides = fixture.options.userOverrides ?? {};
  if (fixture.vueVersion !== undefined) {
    assert(["2", "2.7", "3"].includes(fixture.vueVersion), "invalid Vue version");
    assert(["format_sfc", "format_template"].includes(fixture.api), "non-Vue API version");
  }
  for (const key of Object.keys(fixture.options)) {
    assert(
      ["base", "internalOverrides", "userOverrides"].includes(key),
      `unknown option source: ${key}`,
    );
  }
  assert.equal(fixture.options.base, "FormatOptions::default()");
  assert.deepEqual(
    fixture.options.internalOverrides,
    internal ? { skipScriptStabilization: true } : {},
  );
  assert(overrides && typeof overrides === "object" && !Array.isArray(overrides));
  for (const [key, value] of Object.entries(overrides)) {
    assert(Object.hasOwn(DEFAULT_OPTIONS, key), `unknown formatter option: ${key}`);
    const initial = DEFAULT_OPTIONS[key];
    if (typeof initial === "boolean") assert.equal(typeof value, "boolean");
    else if (typeof initial === "number") {
      assert(
        typeof value === "number" &&
          Number.isInteger(value) &&
          value >= 0 &&
          value <= (key === "tabWidth" ? 255 : 0xffffffff),
      );
    } else if (typeof initial === "string") {
      const enums: Record<string, readonly string[]> = {
        trailingComma: ["none", "es5", "all"],
        arrowParens: ["always", "avoid"],
        endOfLine: ["lf", "crlf", "cr", "auto"],
        quoteProps: ["as-needed", "consistent", "preserve"],
        attributeSortOrder: ["alphabetical", "as-written"],
      };
      assert(
        typeof value === "string" && enums[key]?.includes(value),
        `invalid formatter option: ${key}`,
      );
    } else if (key === "maxAttributesPerLine") {
      assert(
        value === null ||
          (typeof value === "number" &&
            Number.isInteger(value) &&
            value >= 0 &&
            value <= 0xffffffff),
      );
    } else {
      assert(
        value === null ||
          (Array.isArray(value) &&
            value.every(
              (group) => Array.isArray(group) && group.every((item) => typeof item === "string"),
            )),
      );
    }
  }
  const sortingApi = ["GlyphFormatter::format", "format_script_with_sort_imports"].includes(
    fixture.api,
  );
  assert.equal(
    sortingApi,
    fixture.importSorting !== undefined,
    "sorting API needs its actual option probe",
  );
  const sorting = fixture.importSorting;
  if (sorting) {
    assert.deepEqual(Object.keys(sorting).sort(), ["provided", "resolvedOptions", "setting"]);
    assert.equal(typeof sorting.provided, "boolean");
    assert.equal(typeof sorting.resolvedOptions, "string");
    assert.match(
      sorting.resolvedOptions,
      /^(?:Ok\((?:None|Some\(SortImportsOptions \{)|Err\(ScriptFormatError\()/,
    );
    assert(!internal, "import sorting must observe the public stabilization path");
    if (!sorting.provided) assert.equal(sorting.setting, null);
    else assert(sorting.setting !== null, "explicit null is not an import sorting setting");
  }
  const flags = [
    ...(sorting?.provided ? ["--sort-imports", JSON.stringify(sorting.setting)] : []),
    ...(Object.keys(overrides).length ? ["--options", JSON.stringify(overrides)] : []),
    ...(internal ? ["--legacy-single-pass"] : []),
    ...(fixture.vueVersion === undefined ? [] : ["--vue-version", fixture.vueVersion]),
  ];
  return {
    flags,
    effective: {
      ...DEFAULT_OPTIONS,
      ...overrides,
      skipScriptStabilization: internal,
      ...(sorting
        ? {
            sortImportsProvided: sorting.provided,
            sortImports: sorting.setting,
            resolvedSortImports: sorting.resolvedOptions,
          }
        : {}),
      ...(fixture.vueVersion === undefined ? {} : { vueVersion: fixture.vueVersion }),
    },
  };
}

export function assertFormatterError(api: string, internal: boolean, kind: string) {
  const errors: Record<string, readonly string[]> = {
    format_style: ["StyleFormatError"],
    format_json: ["JsonFormatError"],
    format_jsonc: ["JsonFormatError"],
    format_script: ["ScriptParseError", "ScriptFormatError"],
    format_script_with_sort_imports: ["ScriptParseError", "ScriptFormatError"],
    "GlyphFormatter::format": ["ScriptParseError", "ScriptFormatError"],
    format_template: ["TemplateParseError", "TemplateFormatError"],
    format_sfc: [
      "ParseError",
      "ScriptParseError",
      "ScriptFormatError",
      "StyleFormatError",
      "TemplateParseError",
      "TemplateFormatError",
      "JsonFormatError",
    ],
  };
  assert(!internal && errors[api]?.includes(kind), "invalid public formatter error contract");
}

export function formatterApiKind(api: string, kind: string) {
  const kinds: Record<string, string> = {
    format_sfc: "Vue",
    "GlyphFormatter::format": "Vue",
    format_style: "CSS",
    format_json: "JSON",
    format_jsonc: "JSONC",
    format_template: "VueTemplate",
  };
  return ["format_script", "format_script_with_sort_imports"].includes(api)
    ? kind === "JavaScript"
      ? "JavaScript"
      : "TypeScript"
    : kinds[api];
}
