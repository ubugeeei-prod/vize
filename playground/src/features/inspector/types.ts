export type InspectorTarget = "dom" | "ssr";

export interface InspectorOptions {
  customRenderer: boolean;
  vueParserQuirks: boolean;
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
  diff: DiffLine[];
  stats: DiffStats;
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
