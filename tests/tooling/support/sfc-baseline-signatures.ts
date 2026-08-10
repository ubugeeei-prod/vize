import { createHash } from "node:crypto";
import fs from "node:fs";

import type { SfcDialect } from "./sfc-baseline-routes.ts";
import { codePointCompare } from "./sfc-equivalence/blocks.ts";

export type SfcBlock = {
  type?: string;
  lang?: string;
  attrs?: Record<string, string | true>;
  content?: string;
};

export type SfcDescriptor = {
  template?: SfcBlock | null;
  script?: SfcBlock | null;
  scriptSetup?: SfcBlock | null;
  styles?: SfcBlock[];
  customBlocks?: SfcBlock[];
  errors?: unknown[];
};

export function blockSignature(descriptor: SfcDescriptor): unknown {
  const block = (value: SfcBlock | null | undefined, includeContent = false): unknown =>
    value == null
      ? null
      : [
          value.type ?? null,
          value.lang ?? null,
          Object.entries(value.attrs ?? {}).sort(([left], [right]) =>
            codePointCompare(left, right),
          ),
          includeContent ? (value.content ?? "") : null,
        ];
  return {
    template: block(descriptor.template),
    script: block(descriptor.script),
    scriptSetup: block(descriptor.scriptSetup),
    styles: (descriptor.styles ?? []).map((value) => block(value)),
    customBlocks: (descriptor.customBlocks ?? []).map((value) => block(value, true)),
  };
}

export function createBaselineProvenance(
  id: string,
  dialect: SfcDialect,
  packageName: string,
  version: string,
  entry: string,
  normalization: string,
  options: Record<string, unknown>,
) {
  return {
    id,
    dialect,
    package: packageName,
    version,
    entrySha256: sha256(fs.readFileSync(entry)),
    normalization,
    options,
  };
}

export function assertNoCompilerErrors(errors: unknown): void {
  if (!Array.isArray(errors)) throw new Error("compiler result omitted its errors array");
  if (errors.length === 0) return;
  throw new Error(normalizeCompilerMessages(errors).join(" | "));
}

export function normalizeCompilerMessages(messages: unknown[]): string[] {
  return messages.map((message) =>
    typeof message === "string" ? message.replace(/\s+/g, " ").trim() : canonicalJson(message),
  );
}

export function semanticSha256(signature: string): string {
  return sha256(signature);
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value != null && typeof value === "object") {
    const record = value as Record<string, unknown>;
    return `{${Object.keys(record)
      .sort(codePointCompare)
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(record[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value) ?? JSON.stringify(String(value));
}
