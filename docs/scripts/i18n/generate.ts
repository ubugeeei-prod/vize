import { access, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
import { parse, stringify } from "yaml";

const ACCEPT_FLAG = "--accept-machine-translation";
const RESUME_FLAG = "--resume";
const NORMALIZE_ONLY_FLAG = "--normalize-only";
const GOOGLE_TRANSLATE_ENDPOINT = "https://translate.googleapis.com/translate_a/single";
const EDGE_AUTH_ENDPOINT = "https://edge.microsoft.com/translate/auth";
const EDGE_TRANSLATE_ENDPOINT =
  "https://api-edge.cognitive.microsofttranslator.com/translate?api-version=3.0";
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
const translationProvider = providerOption?.slice("--provider=".length) ?? "edge";
if (!new Set(["edge", "google", "argos"]).has(translationProvider)) {
  throw new Error(`Unsupported translation provider: ${translationProvider}`);
}
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
const MAX_TRANSLATION_CHARS = 3_500;
const UNIT_CONCURRENCY = 4;
const FILE_CONCURRENCY = 6;
const REQUEST_INTERVAL_MS = translationProvider === "google" ? 250 : 100;
const translationCache = new Map();
let nextRequestAt = 0;
let rateLimitedUntil = 0;
let requestQueue = Promise.resolve();
const argosPython = process.env.VIZE_I18N_ARGOS_PYTHON;
let edgeTokenPromise;
const edgeBatchQueue = [];
let edgeBatchTimer;
let nextEdgeRequestId = 0;

const normalizeOnly = process.argv.includes(NORMALIZE_ONLY_FLAG);
if (!normalizeOnly && !process.argv.includes(ACCEPT_FLAG)) {
  throw new Error(
    `Translation generation calls an external machine-translation service. Re-run with ${ACCEPT_FLAG}.`,
  );
}

const resume = process.argv.includes(RESUME_FLAG);

function createArgosClient(python) {
  const script = `
import json
import sys
import argostranslate.translate

LOCALE_MAP = {"zh-CN": "zh", "pt-BR": "pt"}
for line in sys.stdin:
    request = json.loads(line)
    try:
        target = LOCALE_MAP.get(request["locale"], request["locale"])
        translated = argostranslate.translate.translate(request["text"], "en", target)
        response = {"id": request["id"], "translated": translated}
    except Exception as error:
        response = {"id": request["id"], "error": str(error)}
    print(json.dumps(response, ensure_ascii=False), flush=True)
`;
  const child = spawn(python, ["-u", "-c", script], { stdio: ["pipe", "pipe", "inherit"] });
  const pending = new Map();
  let nextId = 0;
  createInterface({ input: child.stdout }).on("line", (line) => {
    const response = JSON.parse(line);
    const request = pending.get(response.id);
    if (!request) return;
    pending.delete(response.id);
    if (response.error) request.reject(new Error(response.error));
    else request.resolve(response.translated);
  });
  child.on("exit", (code) => {
    for (const request of pending.values()) {
      request.reject(new Error(`Argos translation process exited with code ${code}`));
    }
    pending.clear();
  });

  return {
    translate(text, locale) {
      return new Promise((resolveTranslation, rejectTranslation) => {
        const id = nextId;
        nextId += 1;
        pending.set(id, { resolve: resolveTranslation, reject: rejectTranslation });
        child.stdin.write(`${JSON.stringify({ id, text, locale })}\n`);
      });
    },
    close() {
      child.stdin.end();
    },
  };
}

if (translationProvider === "argos" && !argosPython) {
  throw new Error("The argos provider requires VIZE_I18N_ARGOS_PYTHON.");
}
const argosClient = translationProvider === "argos" ? createArgosClient(argosPython) : null;

function getEdgeToken(forceRefresh = false) {
  if (!edgeTokenPromise || forceRefresh) {
    edgeTokenPromise = fetch(EDGE_AUTH_ENDPOINT).then(async (response) => {
      if (!response.ok) {
        throw new Error(`Edge translation authentication failed with HTTP ${response.status}`);
      }
      return response.text();
    });
  }
  return edgeTokenPromise;
}

async function translateWithEdge(texts, locale, forceRefresh = false) {
  const targetLocale = { "zh-CN": "zh-Hans", "pt-BR": "pt" }[locale] ?? locale;
  const token = await getEdgeToken(forceRefresh);
  await waitForRequestSlot();
  const url = `${EDGE_TRANSLATE_ENDPOINT}&textType=html&from=${SOURCE_LOCALE}&to=${encodeURIComponent(targetLocale)}`;
  const response = await fetch(url, {
    method: "POST",
    headers: {
      authorization: `Bearer ${token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify(texts.map((text) => ({ Text: text }))),
  });
  if (response.status === 401 && !forceRefresh) {
    return translateWithEdge(texts, locale, true);
  }
  if (!response.ok) {
    if (response.status === 429) {
      const retryAfterHeader = response.headers.get("retry-after");
      const retryAfter = retryAfterHeader ? Number(retryAfterHeader) : Number.NaN;
      const backoff = Number.isFinite(retryAfter) ? retryAfter * 1_000 : 30_000;
      rateLimitedUntil = Math.max(rateLimitedUntil, Date.now() + backoff);
    }
    throw new Error(`Edge translation request failed with HTTP ${response.status}`);
  }
  const payload = await response.json();
  return payload.map((result) => result.translations[0].text);
}

function scheduleEdgeBatch() {
  if (edgeBatchTimer) return;
  edgeBatchTimer = setTimeout(async () => {
    edgeBatchTimer = undefined;
    const locale = edgeBatchQueue[0]?.locale;
    if (!locale) return;

    const batch = [];
    let characterCount = 0;
    for (let index = 0; index < edgeBatchQueue.length && batch.length < 50;) {
      const request = edgeBatchQueue[index];
      const taggedLength = request.text.length + 30;
      if (request.locale !== locale || characterCount + taggedLength > 45_000) {
        index += 1;
        continue;
      }
      edgeBatchQueue.splice(index, 1);
      batch.push(request);
      characterCount += taggedLength;
    }

    try {
      const translations = await translateWithEdge(
        batch.map(
          (request) =>
            `<span class="notranslate">VIZEBATCH${String(request.id).padStart(9, "0")}</span>\n${request.text}`,
        ),
        locale,
      );
      const requestsById = new Map(batch.map((request) => [request.id, request]));
      for (const translated of translations) {
        const marker = translated.match(/VIZEBATCH(\d{9})/);
        const request = marker ? requestsById.get(Number(marker[1])) : undefined;
        if (!marker || !request) {
          throw new Error("Edge translation response lost its batch marker");
        }
        requestsById.delete(request.id);
        request.resolve(
          translated
            .replace(/<span class="notranslate">VIZEBATCH\d{9}<\/span>/, "")
            .replace(marker[0], "")
            .replace(/^\s+/, ""),
        );
      }
      if (requestsById.size > 0) {
        throw new Error("Edge translation response omitted a batched request");
      }
    } catch (error) {
      for (const request of batch) request.reject(error);
    } finally {
      if (edgeBatchQueue.length > 0) scheduleEdgeBatch();
    }
  }, 10);
}

function queueEdgeTranslation(text, locale) {
  return new Promise((resolveTranslation, rejectTranslation) => {
    edgeBatchQueue.push({
      id: nextEdgeRequestId,
      text,
      locale,
      resolve: resolveTranslation,
      reject: rejectTranslation,
    });
    nextEdgeRequestId += 1;
    scheduleEdgeBatch();
  });
}

async function fileExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function waitForRequestSlot() {
  let releaseQueue;
  const previousRequest = requestQueue;
  requestQueue = new Promise((resolveQueue) => {
    releaseQueue = resolveQueue;
  });
  await previousRequest;

  const waitUntil = Math.max(nextRequestAt, rateLimitedUntil);
  const delay = waitUntil - Date.now();
  if (delay > 0) {
    await new Promise((resolveDelay) => setTimeout(resolveDelay, delay));
  }
  nextRequestAt = Date.now() + REQUEST_INTERVAL_MS;
  releaseQueue();
}

function isInsideContentDir(path) {
  const pathFromContent = relative(CONTENT_DIR, path);
  return pathFromContent && !pathFromContent.startsWith(`..${sep}`) && pathFromContent !== "..";
}

async function collectMarkdownFiles(dir = CONTENT_DIR) {
  const files = [];
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

function protectMarkdown(text) {
  const protectedValues = [];
  const markdownPrefixes = [];
  const markdownTableIndents = [];
  const protect = (value) => {
    const token = `VIZEI18NTOKEN${String(protectedValues.length).padStart(5, "0")}Z`;
    protectedValues.push(value);
    return token;
  };

  let protectedText = text;
  protectedText = protectedText.replace(/(`+)([\s\S]*?)\1/g, protect);
  protectedText = protectedText.replace(/<[^>\n]+>/g, protect);
  protectedText = protectedText.replace(/!?\[[^\]\n]+\]\([^)\n]+\)/g, protect);
  protectedText = protectedText.replace(/https?:\/\/[^\s)>]+/g, protect);
  if (translationProvider === "edge") {
    protectedText = protectedText
      .replace(/\*\*([^*\n]+)\*\*/g, '<strong data-vize-markdown="strong">$1</strong>')
      .replace(/__([^_\n]+)__/g, '<strong data-vize-markdown="strong">$1</strong>')
      .replace(/~~([^~\n]+)~~/g, '<del data-vize-markdown="strike">$1</del>')
      .replace(/(?<!\*)\*([^*\n]+)\*(?!\*)/g, '<em data-vize-markdown="emphasis">$1</em>')
      .replace(/(?<!_)_([^_\n]+)_(?!_)/g, '<em data-vize-markdown="emphasis">$1</em>');
    protectedText = protectedText.replace(/^([ \t]*)\|(.+)\|[ \t]*$/gm, (_match, indent, row) => {
      const id = markdownTableIndents.length;
      markdownTableIndents.push(indent);
      const cells = row
        .split("|")
        .map((cell) => `<td>${cell}</td>`)
        .join("");
      return `<table data-vize-markdown-row="${id}"><tr>${cells}</tr></table>`;
    });
    protectedText = protectedText.replace(
      /^(\s{0,3}#{1,6}\s+|\s*(?:[-+*]|\d+[.)])\s+|\s*>+\s*)(.*)$/gm,
      (_match, prefix, content) => {
        const id = markdownPrefixes.length;
        markdownPrefixes.push(prefix);
        return `<p data-vize-markdown-prefix="${id}">${content}</p>`;
      },
    );
  } else {
    protectedText = protectedText.replace(/^(\s{0,3}#{1,6}\s+)/gm, protect);
    protectedText = protectedText.replace(/^(\s*(?:[-+*]|\d+[.)])\s+)/gm, protect);
    protectedText = protectedText.replace(/^(\s*>+\s*)/gm, protect);
  }
  protectedText = protectedText.replace(/[|]/g, protect);
  protectedText = protectedText.replace(/(?:\*\*|__|~~|(?<!\*)\*(?!\*)|(?<!_)_(?!_))/g, protect);
  if (translationProvider === "edge") {
    protectedText = protectedText.replace(/\n/g, protect);
    for (let index = 0; index < protectedValues.length; index += 1) {
      const token = `VIZEI18NTOKEN${String(index).padStart(5, "0")}Z`;
      protectedText = protectedText.replaceAll(token, `<span class="notranslate">${token}</span>`);
    }
  }

  return {
    text: protectedText,
    restore(translated) {
      let restored = translated.replace(
        /<span class="notranslate">(VIZEI18NTOKEN\d+Z)<\/span>/g,
        "$1",
      );
      for (let index = 0; index < protectedValues.length; index += 1) {
        const token = `VIZEI18NTOKEN${String(index).padStart(5, "0")}Z`;
        restored = restored.replaceAll(token, protectedValues[index]);
      }
      restored = restored.replace(/VIZEI18NTOKEN0*(\d{5})Z?/g, (token, index) => {
        return protectedValues[Number(index)] ?? token;
      });
      return restored
        .replace(
          /<table data-vize-markdown-row="(\d+)"><tr>([\s\S]*?)<\/tr><\/table>/g,
          (_match, id, row) => {
            const cells = [...row.matchAll(/<td>([\s\S]*?)<\/td>/g)].map((cell) => cell[1].trim());
            return `${markdownTableIndents[Number(id)] ?? ""}| ${cells.join(" | ")} |`;
          },
        )
        .replace(
          /<p data-vize-markdown-prefix="(\d+)">([\s\S]*?)<\/p>/g,
          (_match, id, content) => `${markdownPrefixes[Number(id)] ?? ""}${content}`,
        )
        .replace(/<strong data-vize-markdown="strong">([\s\S]*?)<\/strong>/g, "**$1**")
        .replace(/<del data-vize-markdown="strike">([\s\S]*?)<\/del>/g, "~~$1~~")
        .replace(/<em data-vize-markdown="emphasis">([\s\S]*?)<\/em>/g, "*$1*")
        .replaceAll("] (", "](");
    },
  };
}

function normalizeTranslatedMarkdown(markdown) {
  return markdown
    .replace(/^- --$/gm, "---")
    .replace(/^([＃]+)/gm, (hashes) => "#".repeat(hashes.length))
    .replaceAll("！[", "![")
    .replace(/【([^】\n]+)】（([^）\n]+)）/g, "[$1]($2)")
    .replace(/【([^】\n]+)】\(([^)\n]+)\)/g, "[$1]($2)")
    .replace(/\[([^\]\n]+)\]（([^）\n]+)）/g, "[$1]($2)")
    .replace(/\[([^\]\n]+)】（([^）\n]+)）/g, "[$1]($2)")
    .replace(/(!?\[)\s+/g, "$1")
    .replace(/\s+\]\(([^)\n]+)\)/g, "]($1)")
    .replace(/^(\s*[-+*])(?=[^-+*\s])/gm, "$1 ");
}

function normalizeMarkdownDocument(markdown) {
  return markdownBlocks(markdown)
    .map((block) => (block.translate ? normalizeTranslatedMarkdown(block.value) : block.value))
    .join("\n");
}

async function translateRequest(text, locale) {
  const key = `${locale}\u0000${text}`;
  const cached = translationCache.get(key);
  if (cached) return cached;

  const request = (async () => {
    if (argosClient) {
      return argosClient.translate(text, locale);
    }

    let lastError;
    for (let attempt = 0; attempt < 5; attempt += 1) {
      try {
        if (translationProvider === "edge") {
          return await queueEdgeTranslation(text, locale);
        }

        await waitForRequestSlot();
        const body = new URLSearchParams({
          client: "gtx",
          sl: SOURCE_LOCALE,
          tl: locale,
          dt: "t",
          q: text,
        });
        const response = await fetch(GOOGLE_TRANSLATE_ENDPOINT, {
          method: "POST",
          headers: { "content-type": "application/x-www-form-urlencoded;charset=UTF-8" },
          body,
        });
        if (!response.ok) {
          if (response.status === 429) {
            const retryAfterHeader = response.headers.get("retry-after");
            const retryAfter = retryAfterHeader ? Number(retryAfterHeader) : Number.NaN;
            const backoff = Number.isFinite(retryAfter) ? retryAfter * 1_000 : 60_000;
            rateLimitedUntil = Math.max(rateLimitedUntil, Date.now() + backoff);
          }
          throw new Error(`translation request failed with HTTP ${response.status}`);
        }

        const payload = await response.json();
        return payload[0].map((part) => part[0]).join("");
      } catch (error) {
        lastError = error;
        await new Promise((resolveDelay) => setTimeout(resolveDelay, 1_000 * 2 ** attempt));
      }
    }
    throw lastError;
  })();

  translationCache.set(key, request);
  return request;
}

async function translateUnit(text, locale) {
  if (!/[A-Za-z]{2}/.test(text) || /^\s*[|: -]+\s*$/.test(text)) return text;

  const protectedMarkdown = protectMarkdown(text);
  const translated = await translateRequest(protectedMarkdown.text, locale);
  return protectedMarkdown.restore(translated);
}

function splitLongBlock(block) {
  if (block.length <= MAX_TRANSLATION_CHARS) return [block];

  const lines = block.split("\n");
  const chunks = [];
  let chunk = "";
  for (const line of lines) {
    if (line.length > MAX_TRANSLATION_CHARS) {
      if (chunk) chunks.push(chunk);
      chunks.push(line);
      chunk = "";
      continue;
    }

    const candidate = chunk ? `${chunk}\n${line}` : line;
    if (candidate.length > MAX_TRANSLATION_CHARS) {
      chunks.push(chunk);
      chunk = line;
    } else {
      chunk = candidate;
    }
  }
  if (chunk) chunks.push(chunk);
  return chunks;
}

function markdownBlocks(markdown) {
  const blocks = [];
  const lines = markdown.split("\n");
  let textLines = [];
  let codeLines = [];
  let fence = null;

  const flushText = () => {
    if (textLines.length > 0) {
      blocks.push({ translate: true, value: textLines.join("\n") });
      textLines = [];
    }
  };
  const flushCode = () => {
    if (codeLines.length > 0) {
      blocks.push({ translate: false, value: codeLines.join("\n") });
      codeLines = [];
    }
  };

  for (const line of lines) {
    const fenceMatch = line.match(/^\s*(`{3,}|~{3,})/);
    if (fence) {
      codeLines.push(line);
      if (fenceMatch?.[1][0] === fence[0] && fenceMatch[1].length >= fence.length) {
        fence = null;
        flushCode();
      }
      continue;
    }

    if (fenceMatch) {
      flushText();
      fence = fenceMatch[1];
      codeLines.push(line);
      continue;
    }

    if (line.trim() === "") {
      flushText();
      blocks.push({ translate: false, value: "" });
    } else {
      textLines.push(line);
    }
  }
  flushText();
  flushCode();
  return blocks;
}

async function mapConcurrent(values, concurrency, mapper) {
  const results = Array.from({ length: values.length });
  let nextIndex = 0;
  await Promise.all(
    Array.from({ length: Math.min(concurrency, values.length) }, async () => {
      while (nextIndex < values.length) {
        const index = nextIndex;
        nextIndex += 1;
        results[index] = await mapper(values[index], index);
      }
    }),
  );
  return results;
}

async function translateMarkdown(markdown, locale) {
  const blocks = markdownBlocks(markdown);
  const jobs = [];
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
  const translatedByBlock = new Map();
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

async function translateFrontmatterValue(value, key, locale) {
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

async function translateFrontmatterObject(value, locale) {
  if (!value || typeof value !== "object") return value;
  const translated = {};
  for (const [key, child] of Object.entries(value)) {
    translated[key] = await translateFrontmatterValue(child, key, locale);
  }
  return translated;
}

function localizeEntryLinks(frontmatter, locale) {
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

async function translateDocument(source, locale, sourcePath) {
  const match = source.match(/^---\n([\s\S]*?)\n---\n?([\s\S]*)$/);
  let frontmatter = "";
  let markdown = source;
  if (match) {
    const parsedFrontmatter = parse(match[1]);
    const translatedFrontmatter = await translateFrontmatterObject(parsedFrontmatter, locale);
    localizeEntryLinks(translatedFrontmatter, locale);
    frontmatter = `---\n${stringify(translatedFrontmatter, { lineWidth: 0 }).trimEnd()}\n---\n`;
    markdown = match[2];
  }

  const translatedMarkdown = await translateMarkdown(markdown, locale);
  return `${frontmatter}<!-- Generated translation; source: ${sourcePath} -->\n\n${translatedMarkdown}`;
}

const sourceFiles = await collectMarkdownFiles();
for (const locale of TARGET_LOCALES) {
  const localeDir = resolve(CONTENT_DIR, locale);
  if (!isInsideContentDir(localeDir)) {
    throw new Error(`Refusing to replace an unsafe locale directory: ${localeDir}`);
  }
  if (normalizeOnly) {
    const localizedFiles = [];
    const collectLocalizedFiles = async (dir) => {
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
argosClient?.close();
