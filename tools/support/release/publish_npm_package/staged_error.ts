import { replaceEvery } from "../command_runtime.ts";

/**
 * Recognizes npm's staged-version conflict through package-manager wrapping.
 * Normalize only the diagnostic view; preserve the original process streams.
 */
export function isStagedPublishError(detail: string): boolean {
  let wrapped = detail;
  for (const separator of ["\r", "\n", "\t", "│"]) {
    wrapped = replaceEvery(wrapped, separator, " ");
  }
  const words = wrapped.split(" ").filter((word) => word !== "");
  return words.join(" ").includes("Cannot publish over previously staged version");
}
