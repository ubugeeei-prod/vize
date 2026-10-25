import ThrowableDiagnostic, { escapeMarkdown } from "@parcel/diagnostic";
import type { Config } from "@parcel/types";
import type { SfcCompileOptionsNapi } from "@vizejs/native";

const Diagnostic =
  (ThrowableDiagnostic as unknown as { default?: typeof ThrowableDiagnostic }).default ??
  ThrowableDiagnostic;

const origin = "@vizejs/parcel-transformer";

/** JSON configuration, tracked and invalidated by Parcel. */
export interface VizeParcelOptions {
  compiler?: Pick<
    SfcCompileOptionsNapi,
    | "vapor"
    | "customElements"
    | "templateSyntax"
    | "experimentalInTagComments"
    | "experimentalPatternedTemplate"
    | "experimentalSelfComponent"
    | "experimentalStrictSlotChildren"
    | "experimentalServerScript"
    | "runtimeModuleName"
  >;
  /** Resolve relative template asset URLs through Parcel. Default true. */
  transformAssetUrls?: boolean;
}

const booleans = new Set([
  "vapor",
  "experimentalInTagComments",
  "experimentalPatternedTemplate",
  "experimentalSelfComponent",
  "experimentalStrictSlotChildren",
  "experimentalServerScript",
]);

export function fail(message: string, filePath?: string): never {
  throw new Diagnostic({
    diagnostic: { origin, message: escapeMarkdown(filePath ? `${filePath}: ${message}` : message) },
  });
}

export function validateOptions(value: unknown, filePath?: string): VizeParcelOptions {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    fail("Parcel Vize configuration must be a JSON object.", filePath);
  }
  const options = value as Record<string, unknown>;
  for (const [name, value] of Object.entries(options)) {
    if (name === "transformAssetUrls" && typeof value === "boolean") continue;
    if (name !== "compiler" || !value || typeof value !== "object" || Array.isArray(value)) {
      fail(`Unknown or invalid Parcel Vize option: ${name}.`, filePath);
    }
    for (const [name, entry] of Object.entries(value)) {
      if (booleans.has(name) && typeof entry === "boolean") continue;
      if (name === "runtimeModuleName" && typeof entry === "string" && entry.length) continue;
      if (
        name === "templateSyntax" &&
        typeof entry === "string" &&
        ["standard", "strict", "quirks"].includes(entry)
      )
        continue;
      if (
        name === "customElements" &&
        Array.isArray(entry) &&
        entry.every((tag) => typeof tag === "string")
      )
        continue;
      fail(`Unknown or invalid Parcel Vize compiler option: ${name}.`, filePath);
    }
  }
  return options as VizeParcelOptions;
}

export async function loadConfig({ config }: { config: Config }): Promise<VizeParcelOptions> {
  const result = await config.getConfig<VizeParcelOptions>([".vizerc.parcel.json"], {
    packageKey: origin,
  });
  return result ? validateOptions(result.contents, result.filePath) : {};
}
