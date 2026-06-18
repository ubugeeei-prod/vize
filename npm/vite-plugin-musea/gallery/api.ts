import type { ArtFileInfo } from "../src/types/index.js";
import {
  emptyA11y,
  emptyDocs,
  emptyPalette,
  emptyTokens,
  fetchStaticDetail,
  fetchStaticPayload,
  getStaticPreviewUrl,
  isStaticGallery,
  joinBasePath,
  staticMutationError,
} from "./staticApi";

export interface PaletteControl {
  name: string;
  control: string;
  default_value?: unknown;
  description?: string;
  required: boolean;
  options: Array<{ label: string; value: unknown }>;
  range?: { min: number; max: number; step?: number };
  group?: string;
}

export interface PaletteApiResponse {
  title: string;
  controls: PaletteControl[];
  groups: string[];
  json: string;
  typescript: string;
}

export interface AnalysisApiResponse {
  props: Array<{
    name: string;
    type: string;
    required: boolean;
    default_value?: unknown;
  }>;
  emits: string[];
}

export interface DocApiResponse {
  markdown: string;
  title: string;
  category?: string;
  variant_count: number;
}

export interface A11yViolation {
  id: string;
  impact: "minor" | "moderate" | "serious" | "critical";
  description: string;
  helpUrl: string;
  nodes: number;
}

export interface A11yApiResponse {
  violations: A11yViolation[];
  passes: number;
  incomplete: number;
}

const basePath =
  (window as unknown as { __MUSEA_BASE_PATH__: string }).__MUSEA_BASE_PATH__ ?? "/__musea__";
const sessionToken =
  (window as unknown as { __MUSEA_SESSION_TOKEN__?: string }).__MUSEA_SESSION_TOKEN__ ?? "";

function mutationHeaders(): HeadersInit {
  const headers = new Headers();
  headers.set("Content-Type", "application/json");
  if (sessionToken) {
    headers.set("X-Musea-Session", sessionToken);
  }
  return headers;
}

async function fetchJson<T>(url: string): Promise<T> {
  const res = await fetch(joinBasePath(url));
  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }
  return res.json() as Promise<T>;
}

export async function fetchArts(): Promise<ArtFileInfo[]> {
  if (isStaticGallery) return (await fetchStaticPayload()).arts;
  return fetchJson<ArtFileInfo[]>("/api/arts");
}

export async function fetchArt(artPath: string): Promise<ArtFileInfo> {
  if (isStaticGallery) {
    const art = (await fetchStaticPayload()).arts.find((item) => item.path === artPath);
    if (!art) throw new Error(`Art not found: ${artPath}`);
    return art;
  }
  return fetchJson<ArtFileInfo>(`/api/arts/${encodeURIComponent(artPath)}`);
}

export async function fetchPalette(artPath: string): Promise<PaletteApiResponse> {
  if (isStaticGallery) return (await fetchStaticDetail(artPath)).palette ?? emptyPalette();
  return fetchJson<PaletteApiResponse>(`/api/arts/${encodeURIComponent(artPath)}/palette`);
}

export async function fetchAnalysis(artPath: string): Promise<AnalysisApiResponse> {
  if (isStaticGallery) {
    return (await fetchStaticDetail(artPath)).analysis ?? { props: [], emits: [] };
  }
  return fetchJson<AnalysisApiResponse>(`/api/arts/${encodeURIComponent(artPath)}/analysis`);
}

export async function fetchDocs(artPath: string): Promise<DocApiResponse> {
  if (isStaticGallery) return (await fetchStaticDetail(artPath)).docs ?? emptyDocs();
  return fetchJson<DocApiResponse>(`/api/arts/${encodeURIComponent(artPath)}/docs`);
}

export async function fetchA11y(artPath: string, variantName: string): Promise<A11yApiResponse> {
  if (isStaticGallery) {
    return (await fetchStaticDetail(artPath)).a11y?.[variantName] ?? emptyA11y();
  }
  return fetchJson<A11yApiResponse>(
    `/api/arts/${encodeURIComponent(artPath)}/variants/${encodeURIComponent(variantName)}/a11y`,
  );
}

export function getPreviewUrl(artPath: string, variantName: string): string {
  if (isStaticGallery) {
    const preview = getStaticPreviewUrl(artPath, variantName);
    if (preview) return preview;
  }
  const art = encodeURIComponent(artPath);
  const variant = encodeURIComponent(variantName);
  return `${joinBasePath("/preview")}?art=${art}&variant=${variant}`;
}

export function getBasePath(): string {
  return basePath;
}
export interface VrtResult {
  artPath: string;
  variantName: string;
  viewport: string;
  passed: boolean;
  isNew?: boolean;
  diffPercentage?: number;
  snapshotPath?: string;
  currentPath?: string;
  diffPath?: string;
  error?: string;
}

export interface VrtSummary {
  total: number;
  passed: number;
  failed: number;
  new: number;
}

export interface VrtApiResponse {
  success: boolean;
  summary: VrtSummary;
  results: VrtResult[];
  artifacts?: {
    reportDir: string;
    htmlReportPath: string;
    jsonReportPath: string;
    snapshotDir: string;
    currentDir: string;
    diffDir: string;
  };
}

// Token types
export interface DesignToken {
  value: string | number;
  type?: string;
  description?: string;
  attributes?: Record<string, unknown>;
  $tier?: "primitive" | "semantic";
  $reference?: string;
  $resolvedValue?: string | number;
}

export interface TokenCategory {
  name: string;
  tokens: Record<string, DesignToken>;
  subcategories?: TokenCategory[];
}

export interface TokensMeta {
  filePath: string;
  tokenCount: number;
  primitiveCount: number;
  semanticCount: number;
}

export interface TokensApiResponse {
  categories: TokenCategory[];
  tokenMap: Record<string, DesignToken>;
  meta: TokensMeta;
  error?: string;
}

export interface TokenMutationResponse {
  categories: TokenCategory[];
  tokenMap: Record<string, DesignToken>;
  dependentsWarning?: string[];
}

export async function fetchTokens(): Promise<TokensApiResponse> {
  if (isStaticGallery) return (await fetchStaticPayload()).tokens ?? emptyTokens();
  return fetchJson<TokensApiResponse>("/api/tokens");
}

export async function createToken(
  tokenPath: string,
  token: Omit<DesignToken, "$resolvedValue">,
): Promise<TokenMutationResponse> {
  if (isStaticGallery) throw staticMutationError();
  const res = await fetch(joinBasePath("/api/tokens"), {
    method: "POST",
    headers: mutationHeaders(),
    body: JSON.stringify({ path: tokenPath, token }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || `API error: ${res.status}`);
  }
  return res.json() as Promise<TokenMutationResponse>;
}

export async function updateToken(
  tokenPath: string,
  token: Omit<DesignToken, "$resolvedValue">,
): Promise<TokenMutationResponse> {
  if (isStaticGallery) throw staticMutationError();
  const res = await fetch(joinBasePath("/api/tokens"), {
    method: "PUT",
    headers: mutationHeaders(),
    body: JSON.stringify({ path: tokenPath, token }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || `API error: ${res.status}`);
  }
  return res.json() as Promise<TokenMutationResponse>;
}

export async function deleteToken(tokenPath: string): Promise<TokenMutationResponse> {
  if (isStaticGallery) throw staticMutationError();
  const res = await fetch(joinBasePath("/api/tokens"), {
    method: "DELETE",
    headers: mutationHeaders(),
    body: JSON.stringify({ path: tokenPath }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || `API error: ${res.status}`);
  }
  return res.json() as Promise<TokenMutationResponse>;
}

// Token usage types
export interface TokenUsageMatch {
  line: number;
  lineContent: string;
  property: string;
}

export interface TokenUsageEntry {
  artPath: string;
  artTitle: string;
  artCategory?: string;
  matches: TokenUsageMatch[];
}

export type TokenUsageMap = Record<string, TokenUsageEntry[]>;

export interface ArtSourceResponse {
  source: string;
  path: string;
}

export async function fetchTokenUsage(): Promise<TokenUsageMap> {
  if (isStaticGallery) return (await fetchStaticPayload()).tokenUsage ?? {};
  return fetchJson<TokenUsageMap>("/api/tokens/usage");
}

export async function fetchArtSource(artPath: string): Promise<ArtSourceResponse> {
  if (isStaticGallery) {
    return (await fetchStaticDetail(artPath)).source ?? { source: "", path: artPath };
  }
  return fetchJson<ArtSourceResponse>(`/api/arts/${encodeURIComponent(artPath)}/source`);
}

export async function updateArtSource(
  artPath: string,
  source: string,
): Promise<{ success: boolean }> {
  if (isStaticGallery) throw staticMutationError();
  const res = await fetch(joinBasePath(`/api/arts/${encodeURIComponent(artPath)}/source`), {
    method: "PUT",
    headers: mutationHeaders(),
    body: JSON.stringify({ source }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || `API error: ${res.status}`);
  }
  return res.json() as Promise<{ success: boolean }>;
}

export async function runVrt(artPath?: string, updateSnapshots?: boolean): Promise<VrtApiResponse> {
  if (isStaticGallery) throw staticMutationError();
  const res = await fetch(joinBasePath("/api/run-vrt"), {
    method: "POST",
    headers: mutationHeaders(),
    body: JSON.stringify({ artPath, updateSnapshots }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || `API error: ${res.status}`);
  }
  return res.json() as Promise<VrtApiResponse>;
}
