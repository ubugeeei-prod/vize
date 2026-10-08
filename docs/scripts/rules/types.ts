/** Source-bound contracts shared by the rule catalogue generators. */
export interface RuleMetadata {
  name: string;
  category: string;
  defaultSeverity: string;
  description: string;
  fixable: boolean;
  implementationLine: number;
  implementationPath: string;
  metaType: string;
  presets: readonly string[];
}

export interface RulePageOptions {
  workspaceRoot: string;
  rules: readonly RuleMetadata[];
  groupedRules: ReadonlyMap<string, readonly RuleMetadata[]>;
  sortedCategories: readonly string[];
  categoryLabels: Readonly<Record<string, string>>;
}

export interface CodeExample {
  language: string;
  source: string;
}
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue };
export interface RuleExample {
  bad: CodeExample;
  good: CodeExample;
  evidence: string;
  availability?: string;
  badDiagnostic?: string;
  typeAware?: boolean;
  standaloneScript?: boolean;
  ruleOptions?: Record<string, JsonValue>;
  note?: string;
  noteJa?: string;
  filename?: string;
  badFilename?: string;
  goodFilename?: string;
}
export type RuleExampleOptions = Omit<RuleExample, "bad" | "good" | "evidence">;
export interface LocalizedText {
  en: string;
  ja: string;
}
export type ExplanationRecord = readonly [string, string, string, string, string];
export interface RuleExplanation {
  bad: LocalizedText;
  good: LocalizedText;
}
export interface ProjectExplanation {
  badExplanation?: LocalizedText;
  goodExplanation?: LocalizedText;
}
export type ProjectFiles = Record<string, string>;
export interface ProjectExample extends ProjectExplanation {
  evidence?: string;
  shared?: ProjectFiles;
  bad: ProjectFiles;
  good: ProjectFiles;
  badGraph?: string;
  goodGraph?: string;
}
export interface ContractExample extends ProjectExample {
  note?: LocalizedText;
}
export interface ProjectRuleExample extends ProjectExample {
  severity?: string;
  note?: string;
  noteJa?: string;
}
export interface ProjectPage {
  id: string;
  text: string;
}
export interface RuleProducer {
  path: string;
  severity?: string;
}
export interface CrossRuleMetadata {
  code: string;
  name: string;
  producer: RuleProducer[];
  status: "cli" | "library" | "contract";
  severity: string;
}
