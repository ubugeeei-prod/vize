export interface VrtResult {
  images?: Record<string, string>;
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

export interface VrtArtifacts {
  reportDir: string;
  htmlReportPath: string;
  jsonReportPath: string;
  snapshotDir: string;
  currentDir: string;
  diffDir: string;
}
