/** Keep the module's reserved root file from replacing an authored config. */
import { lstat, open, readFile } from "node:fs/promises";

export async function writeOwnedNuxtConfig(
  file: string,
  content: string,
  writeChanged: (file: string, content: string) => Promise<boolean>,
): Promise<boolean> {
  try {
    const handle = await open(file, "wx");
    try {
      await handle.writeFile(content, "utf8");
    } finally {
      await handle.close();
    }
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "EEXIST") throw error;
  }

  const metadata = await lstat(file);
  if (!metadata.isFile() || metadata.isSymbolicLink()) {
    throw new Error(`Generated oxlint config must be a regular file: ${file}`);
  }
  let owner: unknown;
  try {
    owner = (
      JSON.parse(await readFile(file, "utf8")) as {
        settings?: { vize?: { generatedBy?: unknown } };
      }
    )?.settings?.vize?.generatedBy;
  } catch {
    // An unparseable file has no verifiable module ownership.
  }
  if (owner !== "@vizejs/nuxt") {
    throw new Error(`Refusing to replace authored ${file}; choose a different lint.configFile`);
  }
  return writeChanged(file, content);
}
