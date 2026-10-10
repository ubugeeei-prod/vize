import type { OxlintHtmlOutcome } from "../../../npm/native/index.js";
import type { InstalledCampaignPlan } from "./n8n-installed-authority.ts";

export interface HtmlCampaignPlan extends Omit<InstalledCampaignPlan, "schema"> {
  schema: "vize.oxlint.installed-html-campaign-v1";
  campaignSha256: string;
  includedRoutes: { binding: { pr: 8406; commit: string }; cli: { pr: 8435; commit: string } };
  providers: Array<{
    version: "1.78.0" | "1.86.0";
    installRoot: string;
    packageLockSha256: string;
  }>;
}
export interface HtmlFixture {
  name: string;
  files: Record<string, string>;
  selected: Record<string, string>;
  formats?: string[];
  implicitDefault?: boolean;
  rules?: Record<string, unknown>;
  settings?: { vize: { helpLevel?: string; preset?: string } };
  denyWarnings?: boolean;
  ignored?: boolean;
  badPlugin?: boolean;
  deny?: boolean;
  target?: string;
  original?: boolean;
  agent?: boolean;
  refused?: boolean;
  status?: number;
}
export interface ExpectedPacket {
  error_count: number;
  warning_count: number;
  diagnostics: Array<{
    rule_name: string;
    severity: string;
    message: string;
    start: number;
    end: number;
    help?: string | null;
    labels: Array<{ message: string; start: number; end: number }>;
    fix?: unknown;
  }>;
}
export interface HtmlRow {
  filename: string;
  message: string;
  code: string;
  severity: string;
  labels: Array<{
    label?: string;
    span: { offset: number; length: number; line: number; column: number };
  }>;
}
export interface HtmlEvent {
  kind: string;
  pid: number;
  source: HtmlCampaignPlan["source"];
  binary: string;
  binarySha256: string;
  observerSha256: string;
  outcome?: string;
  result?: OxlintHtmlOutcome;
  args?: unknown[];
  status?: number | null;
  signal?: string | null;
  error?: unknown;
  stdoutBytes?: number[];
  stderrBytes?: number[];
  [field: string]: unknown;
}
export interface HtmlProcess {
  status: number | null;
  signal: string | null;
  error?: Error;
  stdout: string;
  stderr: string;
  events: HtmlEvent[];
}
export interface HtmlCapture {
  schema: string;
  complete: boolean;
  source: unknown;
  observations: unknown[];
  qualified: {
    cases: number;
    formats: string[];
    wholeNativeCalls: number;
    wholeHostPhases: number;
  };
}
