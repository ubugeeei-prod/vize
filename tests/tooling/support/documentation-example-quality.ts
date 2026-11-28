import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";

import { format } from "oxfmt";
import type { MarkdownSourceExample } from "../../../npm/ui/scripts/reference-docs/markdown-source-examples.ts";

const root = path.resolve(import.meta.dirname, "../../..");
const require = createRequire(import.meta.url);

/** Require canonical formatting and warning-free lint for every complete source packet. */
export async function verifyDocumentationExampleQuality(
  examples: readonly MarkdownSourceExample[],
): Promise<void> {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-doc-quality-"));
  try {
    const files: string[] = [];
    for (const [index, example] of examples.entries()) {
      const formatted = await format(example.filename, example.source, { quoteProps: "preserve" });
      assert.deepEqual(formatted.errors, [], `${example.document}:${example.line}`);
      assert.equal(
        formatted.code,
        example.source,
        `${example.document}:${example.line}: formatting`,
      );
      const file = path.join(directory, `${index}.mts`);
      writeFileSync(file, example.source);
      files.push(file);
    }
    const cli = path.join(path.dirname(require.resolve("oxlint/package.json")), "bin/oxlint");
    const result = spawnSync(process.execPath, [cli, "--deny-warnings", ...files], {
      cwd: root,
      encoding: "utf8",
      timeout: 30_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
