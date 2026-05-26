// Compiler Inspector type definitions (Curator)

export interface InspectorSourceFile {
  path: string;
  source: string;
}

export type InspectorGraphNodeKind = "vue" | "typescript" | "javascript" | "other";

export interface InspectorGraphNode {
  path: string;
  kind: InspectorGraphNodeKind;
  isEntry: boolean;
  sourceBytes: number;
  sourceLines: number;
}

export type InspectorGraphEdgeKind = "import" | "dynamic-import" | "component";

export interface InspectorGraphEdge {
  from: string;
  to: string;
  kind: InspectorGraphEdgeKind;
  specifier: string;
}

export interface InspectorGraph {
  nodes: InspectorGraphNode[];
  edges: InspectorGraphEdge[];
}

export type InspectorDiffLineKind = "same" | "remove" | "add";

export interface InspectorDiffLine {
  kind: InspectorDiffLineKind;
  leftLine: number | null;
  rightLine: number | null;
  text: string;
}

export interface InspectorDiffStats {
  additions: number;
  removals: number;
  unchanged: number;
}

export interface InspectorDiff {
  lines: InspectorDiffLine[];
  stats: InspectorDiffStats;
}
