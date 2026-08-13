import fs from "node:fs";
import path from "node:path";

import { detectJsonIndent, type PlannedFile } from "../setup/config.js";
import type { ProjectDetection } from "./detect.js";
import type { FeatureResult } from "./plan-types.js";
import { renderVscodeExtensions, VSCODE_EXTENSION_ID } from "./templates.js";

const EXTENSIONS_FILE = ".vscode/extensions.json";

/**
 * Plans the editor recommendation.
 *
 * `.vscode/extensions.json` is preferred over `code --install-extension` because
 * it is checked in, applies to everyone on the project, and changes nothing on
 * the machine running `init`. An existing file is merged, never replaced: it
 * usually carries the team's other recommendations.
 */
export function planEditorFile(
  detection: ProjectDetection,
  files: PlannedFile[],
  createdFiles: string[],
  updatedFiles: string[],
): FeatureResult {
  const filename = path.join(detection.root, ".vscode", "extensions.json");
  let source: string;
  try {
    source = fs.readFileSync(filename, "utf8");
  } catch {
    files.push({ filename, source: renderVscodeExtensions(2) });
    createdFiles.push(EXTENSIONS_FILE);
    return {
      id: "editor",
      outcome: "configured",
      detail: `writes ${EXTENSIONS_FILE} recommending ${VSCODE_EXTENSION_ID}`,
      snippet: null,
    };
  }

  const merged = mergeRecommendation(source);
  if (merged === null) {
    return {
      id: "editor",
      outcome: "blocked",
      detail: `${EXTENSIONS_FILE} is not a plain JSON object this tool can extend safely`,
      snippet: `"recommendations": ["${VSCODE_EXTENSION_ID}"]`,
    };
  }
  if (merged === source) {
    return {
      id: "editor",
      outcome: "unchanged",
      detail: `${EXTENSIONS_FILE} already recommends ${VSCODE_EXTENSION_ID}`,
      snippet: null,
    };
  }
  files.push({ filename, source: merged });
  updatedFiles.push(EXTENSIONS_FILE);
  return {
    id: "editor",
    outcome: "configured",
    detail: `adds ${VSCODE_EXTENSION_ID} to ${EXTENSIONS_FILE}`,
    snippet: null,
  };
}

/**
 * Adds the recommendation to an existing file, preserving its other keys and its
 * indentation. Returns the input unchanged when the id is already listed, and
 * `null` when the file is not a JSON object with an array of string
 * recommendations.
 */
function mergeRecommendation(source: string): string | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(source);
  } catch {
    return null;
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    return null;
  }
  const document = parsed as Record<string, unknown>;
  const existing = document.recommendations;
  if (existing !== undefined && !isStringArray(existing)) {
    return null;
  }
  const recommendations = existing ?? [];
  if (recommendations.includes(VSCODE_EXTENSION_ID)) {
    return source;
  }
  document.recommendations = [...recommendations, VSCODE_EXTENSION_ID];
  return `${JSON.stringify(document, null, detectJsonIndent(source))}\n`;
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((entry) => typeof entry === "string");
}
