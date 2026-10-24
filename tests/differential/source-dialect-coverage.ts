import path from "node:path";

export type SourceDialectInput = {
  filename: string;
  source: Buffer | string;
  entry?: "script" | "sfc" | "template";
  vueVersion?: number | string | null;
};

export type SourceInspectedDialectCoverage = {
  state: "source-inspected";
  registryRef: null;
  evidence: Array<{
    kind: "source-inspection";
    dialects: string[];
    selector: string;
  }>;
};

const dialectOrder = [
  "vue0-template",
  "vue1-template",
  "vue2-sfc",
  "vue2.7-sfc",
  "vue3-sfc",
  "petite-vue",
  "pug-template",
  "jsx-babel",
  "jsx-vapor",
  "html-template",
  "js",
  "ts",
  "jsx",
  "tsx",
  "vue-quirks",
];

function runtimeExtension(filename: string) {
  const withoutReceiptSuffix = filename.toLowerCase().replace(/\.txt$/, "");
  return path.posix.extname(withoutReceiptSuffix);
}

export function sourceInspectedDialectCoverage(
  inputs: SourceDialectInput[],
  selector: string,
): SourceInspectedDialectCoverage {
  const dialects = new Set<string>();
  for (const input of inputs) {
    const source = input.source.toString();
    const extension = runtimeExtension(input.filename);
    const isSfc =
      input.entry === "sfc" ||
      extension === ".vue" ||
      (input.entry === undefined && /<(?:template|script|style)(?:\s|>)/i.test(source));
    if (isSfc) {
      const version = String(input.vueVersion ?? "3");
      if (version === "2.7") dialects.add("vue2.7-sfc");
      else if (version === "2") dialects.add("vue2-sfc");
      else dialects.add("vue3-sfc");
      if (
        /<template\b[^>]*\blang\s*=\s*(?:"(?:pug|jade)"|'(?:pug|jade)'|(?:pug|jade)(?:\s|>))/i.test(
          source,
        )
      ) {
        dialects.add("pug-template");
      } else if (/<template(?:\s|>)/i.test(source)) {
        dialects.add("html-template");
      }
      for (const match of source.matchAll(/<script\b([^>]*)>/gi)) {
        const attributes = match[1];
        if (/\blang\s*=\s*["']tsx["']/i.test(attributes)) dialects.add("tsx");
        else if (/\blang\s*=\s*["']ts["']/i.test(attributes)) dialects.add("ts");
        else if (/\blang\s*=\s*["']jsx["']/i.test(attributes)) dialects.add("jsx");
        else dialects.add("js");
      }
    } else if (input.entry === "template") {
      dialects.add("html-template");
    } else if (extension === ".tsx") dialects.add("tsx");
    else if (extension === ".jsx") dialects.add("jsx");
    else if (extension === ".ts") dialects.add("ts");
    else if (extension === ".js" || extension === ".mjs" || extension === ".cjs") {
      dialects.add("js");
    }
  }
  const ordered = dialectOrder.filter((dialect) => dialects.has(dialect));
  if (ordered.length === 0) {
    throw new Error(`${selector}: no authored source dialect could be inspected`);
  }
  return {
    state: "source-inspected",
    registryRef: null,
    evidence: [{ kind: "source-inspection", dialects: ordered, selector }],
  };
}
