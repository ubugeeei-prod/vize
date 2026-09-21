import { randomUUID } from "node:crypto";
import { readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { parse, type ParseError } from "jsonc-parser";
import type { VizePackOptions, VizeTaskConfig } from "./types.ts";
import { resolveConfigExport } from "../config.ts";
import { runNative } from "./runner.ts";

/** Keep authored tsconfigs untouched, including maps and project references. */
export async function emitDeclarations(options: VizePackOptions, metadata: VizeTaskConfig) {
  const config = metadata.config
    ? await resolveConfigExport(metadata.config, { command: "check", mode: "production" })
    : undefined;
  const files: string[] = [];
  const copies = new Map<string, string>();
  async function withMaps(file: string): Promise<string> {
    file = path.resolve(file);
    if ((await stat(file)).isDirectory()) file = path.join(file, "tsconfig.json");
    const known = copies.get(file);
    if (known) return known;
    const errors: ParseError[] = [];
    const source = parse(await readFile(file, "utf8"), errors, { allowTrailingComma: true });
    if (errors.length || !source || typeof source !== "object") {
      throw new Error(`Invalid TypeScript config: ${file}`);
    }
    const temporary = path.join(path.dirname(file), `.vize-vp-${randomUUID()}.json`);
    copies.set(file, temporary);
    const references = source.references
      ? await Promise.all(
          source.references.map(async (reference: { path: string }) => ({
            ...reference,
            path: await withMaps(path.resolve(path.dirname(file), reference.path)),
          })),
        )
      : undefined;
    files.push(temporary);
    await writeFile(
      temporary,
      JSON.stringify({
        extends: file,
        compilerOptions: { declaration: true, declarationMap: options.declarationMap },
        references,
      }),
      { flag: "wx", mode: 0o600 },
    );
    return temporary;
  }
  try {
    let tsconfig = options.tsconfig ?? config?.typeChecker?.tsconfig ?? "tsconfig.json";
    if (options.declarationMap !== undefined) tsconfig = await withMaps(tsconfig);
    const args = ["--declaration", "--tsconfig", tsconfig];
    if (options.declarationDir)
      args.push("--declaration-dir", path.resolve(options.declarationDir));
    const status = await runNative("check", args, metadata);
    if (status !== 0) throw new Error(`Vize declaration generation failed (exit ${status})`);
  } finally {
    await Promise.all(files.map((file) => rm(file, { force: true })));
  }
}
