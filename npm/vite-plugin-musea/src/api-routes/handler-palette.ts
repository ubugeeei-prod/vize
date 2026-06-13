/**
 * Palette handler for the Musea gallery API.
 *
 * Handles GET /api/arts/:path/palette endpoint.
 */

import fs from "node:fs";

import type { ApiRoutesContext, SendJson, SendError } from "./index.js";
import { allowedSourceRoots, resolveComponentSourcePath } from "../component-source.js";
import { loadNative, analyzeSfcFallback } from "../native-loader.js";
import { decodeUrlComponent } from "../security.js";

type PaletteControl = {
  name: string;
  control: string;
  default_value?: unknown;
  description?: string;
  required: boolean;
  options: Array<{ label: string; value: unknown }>;
  range?: { min: number; max: number; step?: number };
  group?: string;
};

type PaletteResponse = {
  title: string;
  controls: PaletteControl[];
  groups: string[];
  json: string;
  typescript: string;
};

type SfcProp = {
  name: string;
  type: string;
  required: boolean;
  default_value?: unknown;
};

/** GET /api/arts/:path/palette */
export async function handleArtPalette(
  ctx: ApiRoutesContext,
  match: RegExpMatchArray,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  const artPath = decodeUrlComponent(match[1], "art path");
  const art = ctx.artFiles.get(artPath);
  if (!art) {
    sendError("Art not found", 404);
    return;
  }

  try {
    const source = await fs.promises.readFile(artPath, "utf-8");
    const binding = loadNative();
    let palette: PaletteResponse;
    if (binding.generateArtPalette) {
      palette = binding.generateArtPalette(source, {
        filename: artPath,
      });
    } else {
      palette = {
        title: art.metadata.title,
        controls: [],
        groups: [],
        json: "{}",
        typescript: "",
      };
    }

    const resolvedComponentPath = resolveComponentSourcePath(
      art,
      artPath,
      allowedSourceRoots(ctx.config.root, ctx.scanRoots),
    );
    if (!resolvedComponentPath) {
      sendJson(palette);
      return;
    }

    try {
      const componentSource = await fs.promises.readFile(resolvedComponentPath, "utf-8");
      const analysis = binding.analyzeSfc
        ? binding.analyzeSfc(componentSource, {
            filename: resolvedComponentPath,
          })
        : analyzeSfcFallback(componentSource, {
            filename: resolvedComponentPath,
          });

      if (analysis.props.length > 0) {
        mergeSfcPropsIntoPalette(palette, analysis.props);
      }
    } catch {
      // Ignore errors reading component file.
    }

    sendJson(palette);
  } catch (e) {
    sendError(e instanceof Error ? e.message : String(e));
  }
}

function mergeSfcPropsIntoPalette(palette: PaletteResponse, props: SfcProp[]): void {
  const controlsByName = new Map<string, PaletteControl>();
  for (const control of palette.controls) {
    controlsByName.set(normalizePropName(control.name), control);
  }

  for (const prop of props) {
    const control = controlsByName.get(normalizePropName(prop.name));
    if (control) {
      applyPropMetadata(control, prop);
    } else {
      palette.controls.push(controlFromProp(prop));
    }
  }

  palette.json = JSON.stringify({ title: palette.title, controls: palette.controls }, null, 2);
  palette.typescript = generateTypescript(palette);
}

function applyPropMetadata(control: PaletteControl, prop: SfcProp): void {
  const inferred = inferControl(prop.type);
  control.name = prop.name;
  control.required = prop.required;
  if (control.default_value === undefined && prop.default_value !== undefined) {
    control.default_value = parseDefault(prop.default_value);
  }
  if (inferred.control !== "text" || control.control === "text") {
    control.control = inferred.control;
  }
  if (inferred.options.length > 0) {
    control.options = inferred.options;
  }
}

function controlFromProp(prop: SfcProp): PaletteControl {
  const inferred = inferControl(prop.type);
  return {
    name: prop.name,
    control: inferred.control,
    default_value: prop.default_value === undefined ? undefined : parseDefault(prop.default_value),
    description: undefined,
    required: prop.required,
    options: inferred.options,
    range: undefined,
    group: undefined,
  };
}

function inferControl(type: string): Pick<PaletteControl, "control" | "options"> {
  const normalized = type.trim();
  const options = literalOptions(normalized);
  if (options.length > 0) return { control: "select", options };
  if (normalized === "boolean") return { control: "boolean", options: [] };
  if (normalized === "number") return { control: "number", options: [] };
  if (normalized.includes("[]") || normalized.startsWith("Array<")) {
    return { control: "array", options: [] };
  }
  if (normalized.startsWith("{") || normalized.startsWith("Record<")) {
    return { control: "object", options: [] };
  }
  return { control: "text", options: [] };
}

function literalOptions(type: string): Array<{ label: string; value: unknown }> {
  if (!type.includes("|") || type.includes("=>")) return [];
  return type
    .split("|")
    .map((part) => part.trim())
    .map((part) => {
      const stringMatch = part.match(/^["']([^"']+)["']$/);
      if (stringMatch) return stringMatch[1];
      if (part === "true") return true;
      if (part === "false") return false;
      const numberValue = Number(part);
      return Number.isFinite(numberValue) ? numberValue : undefined;
    })
    .filter((value): value is string | number | boolean => value !== undefined)
    .map((value) => ({ label: String(value), value }));
}

function parseDefault(value: unknown): unknown {
  if (typeof value !== "string") return value;
  if (value === "true") return true;
  if (value === "false") return false;
  if (/^-?\d+(?:\.\d+)?$/.test(value)) return Number(value);
  return value.replace(/^["']|["']$/g, "");
}

function normalizePropName(name: string): string {
  return name.replace(/[-_]/g, "").toLowerCase();
}

function generateTypescript(palette: PaletteResponse): string {
  const fields = palette.controls
    .map((control) => `  ${control.name}${control.required ? "" : "?"}: ${controlTsType(control)};`)
    .join("\n");
  return `export interface ${pascalCase(palette.title)}Props {\n${fields}\n}\n`;
}

function controlTsType(control: PaletteControl): string {
  if (control.control === "boolean") return "boolean";
  if (control.control === "number" || control.control === "range") return "number";
  if (control.control === "array") return "unknown[]";
  if (control.control === "object") return "Record<string, unknown>";
  if (control.control === "select" && control.options.length > 0) {
    return control.options.map((option) => JSON.stringify(option.value)).join(" | ");
  }
  return "string";
}

function pascalCase(value: string): string {
  return value
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}
