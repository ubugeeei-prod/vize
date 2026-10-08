import { createArgosClient } from "./argos.ts";

export type TranslationProvider = "edge" | "google" | "argos";
type EdgeRequest = {
  id: number;
  text: string;
  locale: string;
  resolve: (text: string) => void;
  reject: (error: unknown) => void;
};
const GOOGLE_TRANSLATE_ENDPOINT = "https://translate.googleapis.com/translate_a/single";
const EDGE_AUTH_ENDPOINT = "https://edge.microsoft.com/translate/auth";
const EDGE_TRANSLATE_ENDPOINT =
  "https://api-edge.cognitive.microsofttranslator.com/translate?api-version=3.0";
const SOURCE_LOCALE = "en";

export function createTranslationClient(
  translationProvider: TranslationProvider,
  argosPython: string | undefined,
) {
  const REQUEST_INTERVAL_MS = translationProvider === "google" ? 250 : 100;
  const translationCache = new Map<string, Promise<string>>();
  let nextRequestAt = 0;
  let rateLimitedUntil = 0;
  let requestQueue = Promise.resolve();
  let edgeTokenPromise: Promise<string> | undefined;
  const edgeBatchQueue: EdgeRequest[] = [];
  let edgeBatchTimer: ReturnType<typeof setTimeout> | undefined;
  let nextEdgeRequestId = 0;
  if (translationProvider === "argos" && !argosPython) {
    throw new Error("The argos provider requires VIZE_I18N_ARGOS_PYTHON.");
  }
  const argosClient = translationProvider === "argos" ? createArgosClient(argosPython!) : null;
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

  async function translateWithEdge(
    texts: string[],
    locale: string,
    forceRefresh = false,
  ): Promise<string[]> {
    const targetLocale =
      ({ "zh-CN": "zh-Hans", "pt-BR": "pt" } as Record<string, string>)[locale] ?? locale;
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
    const payload = (await response.json()) as Array<{ translations: Array<{ text: string }> }>;
    return payload.map((result) => result.translations[0].text);
  }

  function scheduleEdgeBatch() {
    if (edgeBatchTimer) return;
    edgeBatchTimer = setTimeout(async () => {
      edgeBatchTimer = undefined;
      const locale = edgeBatchQueue[0]?.locale;
      if (!locale) return;

      const batch: EdgeRequest[] = [];
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

  function queueEdgeTranslation(text: string, locale: string): Promise<string> {
    return new Promise<string>((resolveTranslation, rejectTranslation) => {
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

  async function waitForRequestSlot() {
    let releaseQueue!: () => void;
    const previousRequest = requestQueue;
    requestQueue = new Promise<void>((resolveQueue) => {
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

  async function translateRequest(text: string, locale: string): Promise<string> {
    const key = `${locale}\u0000${text}`;
    const cached = translationCache.get(key);
    if (cached) return cached;

    const request = (async () => {
      if (argosClient) {
        return argosClient.translate(text, locale);
      }

      let lastError: unknown;
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

          const payload = (await response.json()) as Array<Array<Array<string>>>;
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

  return { translate: translateRequest, close: () => argosClient?.close() };
}
