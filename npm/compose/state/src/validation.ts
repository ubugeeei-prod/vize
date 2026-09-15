import type { AnyStateModelDefinition } from "./types.ts";

export const STATE_ERROR = {
  invalidKey: "VIZE_STATE_INVALID_KEY",
  invalidSource: "VIZE_STATE_SOURCE_NOT_VUE",
  invalidVersion: "VIZE_STATE_INVALID_VERSION",
  missingCommand: "VIZE_STATE_UNKNOWN_COMMAND",
} as const;

export function validateModel(
  model: Pick<AnyStateModelDefinition, "key" | "source" | "version">,
): void {
  if (!model.key) throw new Error(`[${STATE_ERROR.invalidKey}] State model key is required`);
  if (!model.source.endsWith(".vue")) {
    throw new Error(`[${STATE_ERROR.invalidSource}] State model ${model.key} must use .vue`);
  }
  if (!Number.isInteger(model.version) || model.version < 1) {
    throw new Error(
      `[${STATE_ERROR.invalidVersion}] State model ${model.key} version must be >= 1`,
    );
  }
}
