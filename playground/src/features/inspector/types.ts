import type { CrossFileDiagnostic, CrossFileStats } from "../../wasm/index";

export type InspectorTarget = "dom" | "ssr" | "vapor";

export interface InspectorOptions {
  customRenderer: boolean;
  templateSyntax: "standard" | "strict" | "quirks";
}

export interface InspectorFile {
  path: string;
  source: string;
}

export interface InspectorPayload {
  version: 1;
  target: InspectorTarget;
  files: InspectorFile[];
  selectedFile?: string;
  options?: Partial<InspectorOptions>;
}

export interface CompilerRun {
  label: string;
  code: string;
  formattedCode: string;
  parser: "babel" | "typescript";
  warnings: string[];
  error: string | null;
  timeMs: number;
}

export interface InspectorReport {
  filename: string;
  target: InspectorTarget;
  official: CompilerRun;
  vize: CompilerRun;
  virtualTs: CompilerRun;
  vir: CompilerRun;
  graph: InspectorGraphRun;
  diff: DiffLine[];
  stats: DiffStats;
}

export type InspectorGraphNodeKind = "vue" | "typescript" | "javascript" | "other";

export interface InspectorGraphNode {
  path: string;
  kind: InspectorGraphNodeKind;
  isEntry: boolean;
  sourceLines: number;
  sourceBytes: number;
  issueCount: number;
}

export type InspectorGraphEdgeKind = "import" | "dynamic-import" | "component";

export interface InspectorGraphEdge {
  from: string;
  to: string;
  kind: InspectorGraphEdgeKind;
  specifier: string;
}

export interface InspectorGraphRun {
  nodes: InspectorGraphNode[];
  edges: InspectorGraphEdge[];
  diagnostics: CrossFileDiagnostic[];
  circularDependencies: string[][];
  stats: CrossFileStats | null;
  error: string | null;
  timeMs: number;
}

export type DiffLineKind = "same" | "remove" | "add";

export interface DiffLine {
  kind: DiffLineKind;
  leftLine: number | null;
  rightLine: number | null;
  text: string;
}

export interface DiffStats {
  additions: number;
  removals: number;
  unchanged: number;
}
