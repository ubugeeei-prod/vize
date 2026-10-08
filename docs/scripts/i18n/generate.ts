import { access, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { createTranslationClient, type TranslationProvider } from "./client.ts";
import { mapConcurrent } from "./concurrency.ts";
import { normalizeMarkdownDocument } from "./markdown.ts";
import { createDocumentTranslator } from "./translation.ts";

const ACCEPT_FLAG = "--accept-machine-translation";
const RESUME_FLAG = "--resume";
const NORMALIZE_ONLY_FLAG = "--normalize-only";
const CONTENT_DIR = resolve(import.meta.dirname, "../../content");
const SOURCE_LOCALE = "en";
const ALL_TARGET_LOCALES = ["ja", "zh-CN", "pt-BR", "fr"];
const localeOption = process.argv.find((argument) => argument.startsWith("--locales="));
const TARGET_LOCALES = localeOption
  ? localeOption.slice("--locales=".length).split(",").filter(Boolean)
  : ALL_TARGET_LOCALES;
const unknownLocales = TARGET_LOCALES.filter((locale) => !ALL_TARGET_LOCALES.includes(locale));
if (unknownLocales.length > 0) {
  throw new Error(`Unsupported target locale(s): ${unknownLocales.join(", ")}`);
}
const LOCALE_DIRS = new Set([SOURCE_LOCALE, ...ALL_TARGET_LOCALES, "i18n"]);
const providerOption = process.argv.find((argument) => argument.startsWith("--provider="));
const translationProvider = (providerOption?.slice("--provider=".length) ??
  "edge") as TranslationProvider;
if (!new Set(["edge", "google", "argos"]).has(translationProvider)) {
  throw new Error(`Unsupported translation provider: ${translationProvider}`);
}
const FILE_CONCURRENCY = 6;
const argosPython = process.env.VIZE_I18N_ARGOS_PYTHON;

const normalizeOnly = process.argv.includes(NORMALIZE_ONLY_FLAG);
if (!normalizeOnly && !process.argv.includes(ACCEPT_FLAG)) {
  throw new Error(
    `Translation generation calls an external machine-translation service. Re-run with ${ACCEPT_FLAG}.`,
  );
}

const resume = process.argv.includes(RESUME_FLAG);
const translationClient = createTranslationClient(translationProvider, argosPython);
const translateDocument = createDocumentTranslator(
  translationProvider,
  translationClient.translate,
);

async function fileExists(path: string) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function isInsideContentDir(path: string) {
  const pathFromContent = relative(CONTENT_DIR, path);
  return pathFromContent && !pathFromContent.startsWith(`..${sep}`) && pathFromContent !== "..";
}

async function collectMarkdownFiles(dir = CONTENT_DIR): Promise<string[]> {
  const files: string[] = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    if (dir === CONTENT_DIR && entry.isDirectory() && LOCALE_DIRS.has(entry.name)) {
      continue;
    }

    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectMarkdownFiles(path)));
    } else if (entry.isFile() && entry.name.endsWith(".md")) {
      files.push(path);
    }
  }
  return files.sort((left, right) => left.localeCompare(right));
}

const sourceFiles = await collectMarkdownFiles();
for (const locale of TARGET_LOCALES) {
  const localeDir = resolve(CONTENT_DIR, locale);
  if (!isInsideContentDir(localeDir)) {
    throw new Error(`Refusing to replace an unsafe locale directory: ${localeDir}`);
  }
  if (normalizeOnly) {
    const localizedFiles: string[] = [];
    const collectLocalizedFiles = async (dir: string): Promise<void> => {
      for (const entry of await readdir(dir, { withFileTypes: true })) {
        const path = resolve(dir, entry.name);
        if (entry.isDirectory()) await collectLocalizedFiles(path);
        else if (entry.isFile() && entry.name.endsWith(".md")) localizedFiles.push(path);
      }
    };
    if (await fileExists(localeDir)) await collectLocalizedFiles(localeDir);
    for (const localizedFile of localizedFiles) {
      const source = await readFile(localizedFile, "utf8");
      await writeFile(localizedFile, normalizeMarkdownDocument(source), "utf8");
    }
    process.stdout.write(`${locale}: normalized ${localizedFiles.length} files\n`);
    continue;
  }

  if (!resume) {
    await rm(localeDir, { recursive: true, force: true });
  }

  let completed = 0;
  await mapConcurrent(sourceFiles, FILE_CONCURRENCY, async (sourceFile) => {
    const sourcePath = relative(CONTENT_DIR, sourceFile).split(sep).join("/");
    const outputFile = resolve(localeDir, sourcePath);
    if (resume && (await fileExists(outputFile))) {
      completed += 1;
      process.stdout.write(`\r${locale}: ${completed}/${sourceFiles.length}`);
      return;
    }
    await mkdir(resolve(outputFile, ".."), { recursive: true });
    const source = await readFile(sourceFile, "utf8");
    const translated = await translateDocument(source, locale, sourcePath);
    await writeFile(outputFile, translated, "utf8");
    completed += 1;
    process.stdout.write(`\r${locale}: ${completed}/${sourceFiles.length}`);
  });
  process.stdout.write("\n");
}
translationClient.close();
