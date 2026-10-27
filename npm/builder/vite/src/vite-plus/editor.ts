import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import { randomUUID } from "node:crypto";
import { applyEdits, modify, parse, type ParseError } from "jsonc-parser";
import type { LanguageServerConfig } from "../../../../cli/src/types/index.ts";

const recommendations = ["VoidZero.vite-plus-extension-pack", "ubugeeei.vize"];
const defaults: [string[], unknown][] = [
  [["[vue]", "editor.defaultFormatter"], "ubugeeei.vize"],
  [["[vue]", "editor.formatOnSave"], true],
];

export function recommendEditor(
  source: string,
  extensions: boolean,
  languageServer?: LanguageServerConfig,
): string {
  const errors: ParseError[] = [];
  const parsed = parse(source, errors, { allowTrailingComma: true });
  if (errors.length || !parsed || Array.isArray(parsed) || typeof parsed !== "object") {
    throw new Error("Editor setup requires a JSON/JSONC object; existing files were preserved.");
  }
  const formattingOptions = { insertSpaces: true, tabSize: 2 };
  let result = source;
  const set = (keys: (string | number)[], value: unknown) => {
    result = applyEdits(result, modify(result, keys, value, { formattingOptions }));
  };
  if (extensions) {
    if (
      parsed.recommendations !== undefined &&
      (!Array.isArray(parsed.recommendations) ||
        !parsed.recommendations.every((value: unknown) => typeof value === "string"))
    ) {
      throw new Error("extensions.json recommendations must be a string array.");
    }
    const seen = new Set<string>(
      (parsed.recommendations ?? []).map((id: string) => id.toLowerCase()),
    );
    for (const recommendation of recommendations) {
      const id = recommendation.toLowerCase();
      if (!seen.has(id)) {
        // Insert individual IDs so comments inside the authored array survive.
        set(["recommendations", -1], recommendation);
        seen.add(id);
      }
    }
  } else if (languageServer?.enabled !== false && languageServer?.formatting !== false) {
    // Feature switches belong to project LSP policy. Explicit workspace values
    // still win, but setup must not turn inferred defaults into overrides.
    for (const [keys, value] of defaults) {
      const existing = keys.reduce((object, key) => object?.[key], parsed);
      // Preserve explicit workspace decisions, including another Vue formatter.
      if (existing === undefined) set(keys, value);
    }
  }
  return result;
}

export async function setupEditor(
  root = process.cwd(),
  languageServer?: LanguageServerConfig,
): Promise<void> {
  const directory = path.join(root, ".vscode");
  const planned = await Promise.all(
    ["extensions.json", "settings.json"].map(async (name) => {
      const file = path.join(directory, name);
      const source = await readFile(file, "utf8").catch((error: NodeJS.ErrnoException) => {
        if (error.code !== "ENOENT") throw error;
        return "{}\n";
      });
      return { file, source: recommendEditor(source, name === "extensions.json", languageServer) };
    }),
  );
  await mkdir(directory, { recursive: true });
  for (const { file, source } of planned) {
    const temporary = file + "." + randomUUID() + ".tmp";
    try {
      await writeFile(temporary, source, { flag: "wx" });
      await rename(temporary, file);
    } finally {
      await rm(temporary, { force: true });
    }
  }
  console.log(
    "Recommended Vize + Vite Plus extensions and Vue editor defaults. Existing settings were preserved.",
  );
}
