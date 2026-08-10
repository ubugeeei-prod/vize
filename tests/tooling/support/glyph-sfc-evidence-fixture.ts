import { createHash } from "node:crypto";

export const hash = (value: string): string => createHash("sha256").update(value).digest("hex");

function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    const record = value as Record<string, unknown>;
    return `{${Object.keys(record)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value) as string;
}

export function resign(artifact: Record<string, unknown>): Record<string, unknown> {
  const { sha256: _recorded, ...unsigned } = artifact;
  return { ...unsigned, sha256: hash(canonicalJson(unsigned)) };
}

export function glyphSfcEvidenceInput() {
  const semantic = hash("semantic");
  return {
    sourceCommit: "a".repeat(40),
    formatter: { version: "0.346.0", binarySha256: hash("vize") },
    waiverValidationError: null,
    availableBaselines: [
      ...["0.10", "0.11", "1"].map((dialect) => ({
        id: `unsupported-vue-${dialect}`,
        dialect,
        package: null,
        version: null,
        entrySha256: null,
        normalization: "unavailable",
        options: {},
      })),
      {
        id: "vue2.7",
        dialect: "2.7",
        package: "@vue/compiler-sfc",
        version: "2.7.16",
        entrySha256: hash("compiler-2.7"),
        normalization: "vue2-render-v1",
        options: {
          parse: { pad: false },
          compile: {
            isProduction: true,
            prettify: false,
            compilerOptions: {
              comments: true,
              outputSourceRange: true,
              whitespace: "preserve",
            },
          },
        },
      },
      {
        id: "vue3",
        dialect: "3",
        package: "@vue/compiler-sfc",
        version: "3.6.0-beta.10",
        entrySha256: hash("compiler-3"),
        normalization: "vue3-template-ast-v1",
        options: { sourceMap: false },
      },
    ],
    expectedFiles: [
      {
        project: "gogocode",
        revision: "b".repeat(40),
        path: "src/App.vue",
        routeId: "vue2",
        dialect: "2",
        baselineId: "vue2.6",
      },
    ],
    files: [
      {
        project: "gogocode",
        revision: "b".repeat(40),
        path: "src/App.vue",
        routeId: "vue2",
        dialect: "2",
        baselineId: "vue2.6",
        originalSha256: hash("original"),
        formattedSha256: hash("formatted"),
        beforeSemanticSha256: semantic,
        afterSemanticSha256: semantic,
        verdict: "equivalent",
        reasonCode: null,
        differences: [],
        failure: null,
        waiver: null,
        baseline: {
          id: "vue2.6",
          dialect: "2",
          package: "vue-template-compiler",
          version: "2.6.14",
          entrySha256: hash("compiler"),
          normalization: "vue2-render-v1",
          options: {
            parse: { pad: false },
            compile: { comments: true, outputSourceRange: true, whitespace: "preserve" },
          },
        },
      },
    ],
  };
}
