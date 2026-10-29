import type { FileResult, types as t } from "@babel/core";
import type { VizeBabelMetadata } from "../src/types.ts";

export type BabelCore = typeof import("@babel/core");
export type BabelAst = t.Node;
export type BabelResult = Omit<FileResult, "metadata"> & {
  metadata: Record<string, unknown> & {
    vize?: { predecessor?: string; jsx?: VizeBabelMetadata };
  };
};
export interface WholeError {
  name: unknown;
  constructor: string;
  ownProperties: Record<string, unknown>;
}
export interface SourceMapPacket {
  version: number;
  file?: string;
  names: string[];
  sources: string[];
  sourcesContent?: (string | null)[];
  mappings: string;
  sourceRoot?: string;
}
export interface MappingModule {
  GenMapping: new (options: { file: string }) => object;
  setSourceContent(map: object, source: string, content: string): void;
  addMapping(
    map: object,
    mapping: {
      generated: { line: number; column: number };
      source: string;
      original: { line: number; column: number };
    },
  ): void;
  toEncodedMap(map: object): SourceMapPacket & { file: string; sourcesContent?: string[] };
}
export interface TraceModule {
  TraceMap: new (map: object) => object;
  originalPositionFor(
    map: object,
    position: { line: number; column: number },
  ): {
    source: string | null;
    line: number | null;
    column: number | null;
    name: string | null;
  };
}

export interface HostLoader {
  requireHost: NodeJS.Require;
  core(): Promise<BabelCore>;
  vize(): Promise<
    import("@babel/core").PluginTarget | { default: import("@babel/core").PluginTarget }
  >;
  typescript(): ReturnType<HostLoader["vize"]>;
  modules(): ReturnType<HostLoader["vize"]>;
  decorators(): ReturnType<HostLoader["vize"]>;
  classProperties(): ReturnType<HostLoader["vize"]>;
  react(): ReturnType<HostLoader["vize"]>;
  mapping(): Promise<MappingModule>;
  trace(): Promise<TraceModule>;
  vue(): Promise<{ createSSRApp(options: { render(): unknown }): unknown }>;
  renderer(): Promise<{ renderToString(app: unknown): Promise<string> }>;
}
export type HostWorkerInput = import("./hosts/environment.ts").HostSettings & {
  host: ReturnType<typeof import("./hosts/environment.ts").prepareHost>;
  format: "esm" | "cjs";
  phase: "cold" | "positive";
  report: string;
};
export interface HostObservation {
  id: string;
  input: import("./hosts/fixtures.ts").HostFixture;
  result: BabelResult | null;
  error: WholeError | null;
  assertion: WholeError | null;
  generatorCalls?: number[];
  reservations?: { label: string; name: string }[];
  upstream?: { map: SourceMapPacket; authored: string };
  baseline?: import("@babel/core").FileResult | null;
  html?: string;
}
export interface HostReport {
  schema: string;
  version: number;
  source: string;
  host: HostWorkerInput["host"];
  format: HostWorkerInput["format"];
  phase: HostWorkerInput["phase"];
  tarballSha256: string;
  scope: string;
  observations: HostObservation[];
  resolutions: { specifier: string; parentURL: string | undefined }[];
  fatal: WholeError | null;
  preparation?: Awaited<ReturnType<typeof import("./hosts/environment.ts").activeNative>> | null;
  loadedAddons?: { path: string; sha256: string }[];
}
