import type { JsxCompileResultNapi } from "@vizejs/native";
import type { BabelResult, WholeError } from "../host-types.ts";
import type {
  RuntimeInput,
  NativePreparation,
  CompilerResult,
  RuntimeIdentity,
} from "../runtime/types.ts";

export interface StyleMetrics {
  setup: number;
  clicks: number;
  events: { type: string; trusted: boolean }[];
}
export interface StyleDiagnostics {
  warnings: { message: string; trace: string }[];
  errors: { name: string; message: string; info: string }[];
}
export interface StyleDomTree {
  nodeType: number;
  nodeName: string;
  data: string | null;
  attributes: [string, string][];
  children: StyleDomTree[];
}
export interface StyleSnapshot {
  html: string;
  tree: StyleDomTree[];
  metrics: StyleMetrics;
  diagnostics: StyleDiagnostics;
  sameNodes: boolean;
  styles: {
    color: string;
    backgroundColor: string;
    fontWeight: string;
    fontFamily: string;
    float: string;
    colorPriority: string;
    weightPriority: string;
    customBrand: string;
    customPriority: string;
  };
}
export interface StyleDispose {
  html: string;
  diagnostics: StyleDiagnostics;
}
export interface StyleBrowserApi {
  ready: boolean;
  snapshot(): StyleSnapshot;
  restore(): Promise<void>;
  dispose(): Promise<StyleDispose>;
}
export interface StyleRuntimeModule {
  App: Parameters<typeof import("vue").createVaporApp>[0];
  controls: { reset(): void };
}
export interface StyleObservation {
  kind: "candidate" | "dynamic" | "static";
  snapshots: (StyleSnapshot & { name: string })[];
  console: { type: string; text: string }[];
  pageErrors: WholeError[];
  requestFailures: { url: string; failure: { errorText: string } | null }[];
  dispose: StyleDispose | null;
  error: WholeError | null;
}
export interface StyleBrowserRow {
  id: string;
  role: string;
  observations: StyleObservation[];
  error: WholeError | null;
}
export interface StyleBrowserReport {
  runtime: RuntimeIdentity;
  chromiumVersion: string | undefined;
  scope: string;
  rows: StyleBrowserRow[];
  requests: { pathname: string; method: string | undefined }[];
}
export interface NativeCall {
  text: string;
  options: Parameters<typeof import("@vizejs/native").compileJsx>[1];
  result: JsxCompileResultNapi;
}
export interface StylePacket {
  id: string;
  source: string;
  role: string;
  preparation: NativePreparation;
  result: BabelResult | null;
  native: NativeCall | null | undefined;
  references: {
    kind: string;
    template: string;
    result: CompilerResult;
    errors: string[];
    warnings: string[];
  }[];
  nativeOrigins?: ReturnType<typeof import("./maps.ts").nativeOrigins>;
  loadedAddons?: { path: string; sha256: string }[];
}
export interface StyleReport {
  input: RuntimeInput;
  packets: StylePacket[];
  browser: StyleBrowserReport | null;
  error: WholeError | null;
}
declare global {
  var __styleGuardInput: { module: string };
  var __styleGuardMetrics: StyleMetrics;
  var __styleGuard: StyleBrowserApi | undefined;
  var __styleGuardFailure: { name: string; message: string; stack?: string } | undefined;
}
