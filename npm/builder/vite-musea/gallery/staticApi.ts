import type { ArtFileInfo } from "../src/types/index.js";
import type {
  A11yApiResponse,
  AnalysisApiResponse,
  ArtSourceResponse,
  DocApiResponse,
  PaletteApiResponse,
  TokensApiResponse,
  TokenUsageMap,
} from "./api";

const win = window as unknown as {
  __MUSEA_BASE_PATH__?: string;
  __MUSEA_STATIC__?: boolean;
  __MUSEA_STATIC_PREVIEWS__?: Record<string, Record<string, string>>;
};

const basePath = win.__MUSEA_BASE_PATH__ ?? "/__musea__";
const staticPreviews = win.__MUSEA_STATIC_PREVIEWS__ ?? {};

export const isStaticGallery = Boolean(win.__MUSEA_STATIC__);

export interface StaticGalleryPayload {
  arts: ArtFileInfo[];
  previews: Record<string, Record<string, string>>;
  details: Record<
    string,
    {
      source?: ArtSourceResponse;
      palette?: PaletteApiResponse;
      analysis?: AnalysisApiResponse;
      docs?: DocApiResponse;
      a11y?: Record<string, A11yApiResponse>;
    }
  >;
  tokens?: TokensApiResponse;
  tokenUsage?: TokenUsageMap;
}

let staticPayloadPromise: Promise<StaticGalleryPayload> | null = null;

export function joinBasePath(url: string): string {
  const normalizedBase = basePath === "/" ? "" : basePath.replace(/\/+$/, "");
  return `${normalizedBase}${url.startsWith("/") ? url : `/${url}`}`;
}

export async function fetchStaticPayload(): Promise<StaticGalleryPayload> {
  staticPayloadPromise ??= fetch(joinBasePath("/api/static.json")).then((res) => {
    if (!res.ok) throw new Error(`API error: ${res.status} ${res.statusText}`);
    return res.json() as Promise<StaticGalleryPayload>;
  });
  return staticPayloadPromise;
}

export async function fetchStaticDetail(
  artPath: string,
): Promise<StaticGalleryPayload["details"][string]> {
  const detail = (await fetchStaticPayload()).details[artPath];
  if (!detail) throw new Error(`Art not found: ${artPath}`);
  return detail;
}

export function getStaticPreviewUrl(artPath: string, variantName: string): string | undefined {
  return staticPreviews[artPath]?.[variantName];
}

export function staticMutationError(): Error {
  return new Error("This action is not available in a static Musea gallery.");
}

export function emptyPalette(): PaletteApiResponse {
  return { title: "", controls: [], groups: [], json: "{}", typescript: "" };
}

export function emptyDocs(): DocApiResponse {
  return { markdown: "", title: "", variant_count: 0 };
}

export function emptyA11y(): A11yApiResponse {
  return { violations: [], passes: 0, incomplete: 0 };
}

export function emptyTokens(): TokensApiResponse {
  return {
    categories: [],
    tokenMap: {},
    meta: { filePath: "", tokenCount: 0, primitiveCount: 0, semanticCount: 0 },
  };
}
