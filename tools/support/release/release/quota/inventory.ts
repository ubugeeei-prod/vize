import { createHash } from "node:crypto";

export const CRATE_INVENTORY_PATH = "tools/support/release/publish_crates/main.ts";
function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
/** Read the product source's literal canonical cohort, never a newer tooling cohort. */
export function releaseCrateInventory(source: string): string[] {
  const match = /export const publishedCrates:[^=]+=[\s\S]*?\[([\s\S]*?)\];/.exec(source);
  requireValue(match?.[1] !== undefined, "source-pinned canonical crate inventory unavailable");
  requireValue(
    /^\s*(?:"[a-zA-Z0-9_-]+"\s*,\s*)*"[a-zA-Z0-9_-]+"\s*,?\s*$/.test(match[1]),
    "canonical cohort must contain only literal names",
  );
  const crates = [...match[1].matchAll(/"([^"]+)"/g)].map((row) => row[1]);
  requireValue(
    crates.length > 0 &&
      crates.every((name) => typeof name === "string" && /^[a-zA-Z0-9_-]+$/.test(name)) &&
      new Set(crates).size === crates.length,
    "source-pinned crate inventory invalid or duplicated",
  );
  return crates as string[];
}
export function decodeCrateInventoryFile(value: unknown): { source: string; gitBlob: string } {
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "inventory GitHub object required",
  );
  const file = value as Record<string, unknown>;
  requireValue(
    file.path === CRATE_INVENTORY_PATH &&
      file.type === "file" &&
      file.encoding === "base64" &&
      typeof file.content === "string" &&
      typeof file.sha === "string" &&
      /^[a-f0-9]{40}$/.test(file.sha),
    "inventory GitHub identity differs",
  );
  const bytes = Buffer.from(file.content, "base64");
  const gitBlob = createHash("sha1")
    .update("blob " + bytes.length + "\0")
    .update(bytes)
    .digest("hex");
  requireValue(gitBlob === file.sha, "inventory Git blob checksum mismatch");
  return { source: new TextDecoder("utf-8", { fatal: true }).decode(bytes), gitBlob };
}
