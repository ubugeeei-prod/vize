import type { InputOptions } from "@babel/core";
import type { BabelResult, HostLoader, WholeError } from "../host-types.ts";
import type { RuntimeMetrics } from "./fixtures.ts";

export type RuntimeIdentity = ReturnType<typeof import("./environment.ts").runtimeIdentity>;
export type RuntimeInput = ReturnType<typeof import("./environment.ts").settings> & {
  id: string;
  compiler: ReturnType<typeof import("./environment.ts").prepareCompiler>;
  runtime: RuntimeIdentity;
  format: "esm" | "cjs";
  qualificationRole: string;
  report: string;
};
export type RuntimeLoader = Pick<HostLoader, "requireHost" | "core" | "typescript"> & {
  plugin: HostLoader["vize"];
};
export type NativePreparation = Awaited<
  ReturnType<typeof import("../hosts/environment.ts").activeNative>
>;
export interface CompilePacket {
  source: string;
  phase: "client" | "server";
  target: "vdom" | "vapor";
  coreVersion: string;
  preparation: NativePreparation;
  result: BabelResult | null;
  error: WholeError | null;
  loadedAddons: { path: string; sha256: string }[];
  codeSha256?: string;
}
export interface CompilerResult {
  code: string;
  [key: string]: unknown;
}
export interface TemplateCompiler {
  compile(
    source: string,
    options: {
      mode: "module";
      prefixIdentifiers: boolean;
      vapor: boolean;
      comments: boolean;
      onError(error: unknown): void;
      onWarn(warning: unknown): void;
    },
  ): CompilerResult;
}
export interface GeneratedFile {
  file: string;
  sha256: string;
}
export interface Diagnostics {
  warnings: string[];
  errors: { name: string; message: string; stack?: string }[];
}
export interface DomTree {
  nodeType: number;
  nodeName: string;
  namespace?: string | null;
  attributes?: [string, string][];
  disabled?: boolean;
  style?: { color: string; fontWeight: string };
  data?: string;
  children: DomTree[];
}
export interface RuntimeSnapshot {
  html: string;
  tree: DomTree[];
  metrics: RuntimeMetrics;
  diagnostics: Diagnostics;
}
export interface Hydration {
  mode: "mount" | "hydrate";
  beforeHtml: string;
  beforeTree: DomTree[];
  originalCount: number;
  afterCount: number;
  retainedInOrder: boolean[];
  sameCount: boolean;
}
export interface RuntimeDispose {
  html: string;
  diagnostics: Diagnostics;
}
export interface BrowserInput {
  mode: "mount" | "hydrate";
  kind: "candidate" | "reference";
  version: string;
  target: "vdom" | "vapor";
  module: string;
  html: string;
}
export interface RuntimeModule {
  App: Parameters<typeof import("vue").createApp>[0];
  controls: { update(): void; restore(): void };
}
export interface RuntimeBrowserApi {
  ready: boolean;
  version: string;
  hydration: Hydration;
  snapshot(): Promise<RuntimeSnapshot>;
  update(): Promise<void>;
  restore(): Promise<void>;
  dispose(): RuntimeDispose;
}
export interface ServerObservation {
  kind: "candidate" | "reference";
  html: string | null;
  metrics: RuntimeMetrics | null;
  warnings: string[][];
  errors: unknown[][];
  failure: WholeError | null;
}
export interface ServerReport {
  runtime: RuntimeIdentity;
  observations: ServerObservation[];
  fatal: WholeError | null;
}
export interface BrowserObservation {
  kind: "candidate" | "reference";
  mode: "mount" | "hydrate";
  snapshots: (RuntimeSnapshot & { name: string })[];
  console: {
    type: string;
    text: string;
    location: { url: string; lineNumber: number; columnNumber: number };
  }[];
  pageErrors: WholeError[];
  requestFailures: { url: string; failure: { errorText: string } | null }[];
  hydration: Hydration | null;
  dispose: RuntimeDispose | null;
  failure: WholeError | null;
}
export interface BrowserReport {
  chromiumVersion: string | undefined;
  runtime: RuntimeIdentity;
  observations: BrowserObservation[];
  requests: { pathname: string; method: string | undefined }[];
}
export interface RuntimeReport {
  input: RuntimeInput;
  compiled: CompilePacket[] | null;
  reference: GeneratedFile[] | null;
  server: ServerReport | null;
  browser: BrowserReport | null;
  error: WholeError | null;
  directory?: string;
  preservationError?: WholeError;
}
export type CompleteBabelOptions = InputOptions & { plugins: NonNullable<InputOptions["plugins"]> };

declare global {
  var __vizeBabelRuntime: RuntimeMetrics;
  var __vizeRuntimeInput: BrowserInput;
  var __vizeRuntime: RuntimeBrowserApi | undefined;
  var __vizeHydrationObservation: Hydration | undefined;
  var __vizeRuntimeFailure: { name: string; message: string; stack?: string } | undefined;
}
