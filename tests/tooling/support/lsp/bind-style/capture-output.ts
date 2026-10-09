import fs from "node:fs";
import path from "node:path";

/** Retain earlier evidence while giving this invocation its own capture leaves. */
export function createCaptureOutput(root: string, configuredParent?: string): string {
  const parent =
    configuredParent ?? path.join(root, "target/differential/lsp-bind-style-code-actions");
  fs.mkdirSync(parent, { recursive: true });
  return fs.mkdtempSync(path.join(parent, "invocation-"));
}
