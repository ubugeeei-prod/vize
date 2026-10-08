import { parse, stringify } from "yaml";
import type { TranslationProvider } from "./client.ts";
import { mapConcurrent } from "./concurrency.ts";
import {
  markdownBlocks,
  normalizeTranslatedMarkdown,
  protectMarkdown,
  splitLongBlock,
} from "./markdown.ts";
const UNIT_CONCURRENCY = 4;
const TRANSLATABLE_FRONTMATTER_KEYS = new Set([
  "title",
  "description",
  "text",
  "tagline",
  "alt",
  "details",
  "linkText",
  "body",
]);

export function createDocumentTranslator(
  translationProvider: TranslationProvider,
  translateRequest: (text: string, locale: string) => Promise<string>,
) {
  async function translateUnit(text: string, locale: string) {
    if (!/[A-Za-z]{2}/.test(text) || /^\s*[|: -]+\s*$/.test(text)) return text;

    const protectedMarkdown = protectMarkdown(text, translationProvider);
    const translated = await translateRequest(protectedMarkdown.text, locale);
    return protectedMarkdown.restore(translated);
  }

  async function translateMarkdown(markdown: string, locale: string) {
    const blocks = markdownBlocks(markdown);
    const jobs: Array<{ blockIndex: number; chunkIndex: number; chunk: string }> = [];
    for (const [blockIndex, block] of blocks.entries()) {
      if (!block.translate) continue;
      for (const [chunkIndex, chunk] of splitLongBlock(block.value).entries()) {
        jobs.push({ blockIndex, chunkIndex, chunk });
      }
    }

    const translatedJobs = await mapConcurrent(jobs, UNIT_CONCURRENCY, async (job) => ({
      ...job,
      translated: await translateUnit(job.chunk, locale),
    }));
    const translatedByBlock = new Map<number, string[]>();
    for (const job of translatedJobs) {
      const chunks = translatedByBlock.get(job.blockIndex) ?? [];
      chunks[job.chunkIndex] = job.translated;
      translatedByBlock.set(job.blockIndex, chunks);
    }

    return blocks
      .map((block, blockIndex) => {
        const translated = translatedByBlock.get(blockIndex)?.join("\n") ?? block.value;
        return block.translate ? normalizeTranslatedMarkdown(translated) : translated;
      })
      .join("\n");
  }

  async function translateFrontmatterValue(
    value: unknown,
    key: string,
    locale: string,
  ): Promise<unknown> {
    if (typeof value === "string") {
      if (!TRANSLATABLE_FRONTMATTER_KEYS.has(key)) return value;
      return translateUnit(value, locale);
    }
    if (Array.isArray(value)) {
      return Promise.all(value.map((item) => translateFrontmatterObject(item, locale)));
    }
    if (value && typeof value === "object") {
      return translateFrontmatterObject(value, locale);
    }
    return value;
  }

  async function translateFrontmatterObject(value: unknown, locale: string): Promise<unknown> {
    if (!value || typeof value !== "object") return value;
    const translated: Record<string, unknown> = {};
    for (const [key, child] of Object.entries(value)) {
      translated[key] = await translateFrontmatterValue(child, key, locale);
    }
    return translated;
  }

  function localizeEntryLinks(value: unknown, locale: string) {
    const frontmatter = value as {
      layout?: string;
      hero?: { actions?: Array<{ link?: unknown }> };
      features?: Array<{ link?: unknown }>;
    };
    if (frontmatter.layout !== "entry") return;
    const items = [...(frontmatter.hero?.actions ?? []), ...(frontmatter.features ?? [])];
    for (const item of items) {
      if (
        typeof item.link === "string" &&
        !/^(?:[a-z]+:|\/|#)/i.test(item.link) &&
        !item.link.startsWith(`${locale}/`)
      ) {
        item.link = `${locale}/${item.link}`;
      }
    }
  }

  async function translateDocument(source: string, locale: string, sourcePath: string) {
    const match = source.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
    let frontmatter = "";
    let markdown = source;
    if (match) {
      const parsedFrontmatter: unknown = parse(match[1]);
      const translatedFrontmatter = await translateFrontmatterObject(parsedFrontmatter, locale);
      localizeEntryLinks(translatedFrontmatter, locale);
      frontmatter = `---\n${stringify(translatedFrontmatter, { lineWidth: 0 }).trimEnd()}\n---\n`;
      markdown = match[2];
    }

    const translatedMarkdown = await translateMarkdown(markdown, locale);
    return `${frontmatter}<!-- Generated translation; source: ${sourcePath} -->\n\n${translatedMarkdown}`;
  }

  return translateDocument;
}
