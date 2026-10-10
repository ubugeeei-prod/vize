import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

/** Human-authored and reviewed translations are authoritative generation inputs. */
export function isReviewedTranslation(document: string) {
  return (
    document.includes("<!-- Reviewed translation;") ||
    !document.includes("<!-- Generated translation;")
  );
}

/** Retain complete reviewed documents before the locale directory is rebuilt. */
export async function collectReviewedTranslations(sourcePaths: string[], localeDirectory: string) {
  const documents = new Map<string, string>();
  for (const sourcePath of sourcePaths) {
    const output = resolve(localeDirectory, sourcePath);
    let document: string;
    try {
      document = await readFile(output, "utf8");
    } catch (error) {
      if (error instanceof Error && "code" in error && error.code === "ENOENT") continue;
      throw error;
    }
    if (isReviewedTranslation(document)) documents.set(sourcePath, document);
  }
  return documents;
}
