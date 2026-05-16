export interface CrossFileIssue {
  id: string;
  type: string;
  code: string;
  severity: "error" | "warning" | "info";
  message: string;
  file: string;
  line: number;
  column: number;
  endLine?: number;
  endColumn?: number;
  relatedLocations?: Array<{ file: string; line: number; column: number; message: string }>;
  suggestion?: string;
}
