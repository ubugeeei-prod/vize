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
    let palette: {
      title: string;
      controls: Array<{
        name: string;
        control: string;
        default_value?: unknown;
        description?: string;
        required: boolean;
        options: Array<{ label: string; value: unknown }>;
        range?: { min: number; max: number; step?: number };
        group?: string;
      }>;
      groups: string[];
      json: string;
      typescript: string;
    };
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

    // If the native palette returned no controls, try JS-based SFC analysis
    if (palette.controls.length === 0 && art.metadata.component) {
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
          palette.controls = analysis.props.map((prop) => {
            let control = "text";
            if (prop.type === "boolean") control = "boolean";
            else if (prop.type === "number") control = "number";
            else if (prop.type.includes("|") && !prop.type.includes("=>")) {
              control = "select";
            }

            const options: Array<{ label: string; value: unknown }> = [];
            if (control === "select") {
              const optionMatches = prop.type.match(/"([^"]+)"/g);
              if (optionMatches) {
                for (const opt of optionMatches) {
                  const val = opt.replace(/"/g, "");
                  options.push({ label: val, value: val });
                }
              }
            }

            return {
              name: prop.name,
              control,
              default_value:
                prop.default_value !== undefined
                  ? prop.default_value === "true"
                    ? true
                    : prop.default_value === "false"
                      ? false
                      : typeof prop.default_value === "string" && prop.default_value.startsWith('"')
                        ? prop.default_value.replace(/^"|"$/g, "")
                        : prop.default_value
                  : undefined,
              description: undefined,
              required: prop.required,
              options,
              range: undefined,
              group: undefined,
            };
          });

          palette.json = JSON.stringify(
            { title: palette.title, controls: palette.controls },
            null,
            2,
          );
          palette.typescript = `export interface ${palette.title}Props {\n${palette.controls
            .map(
              (c) =>
                `  ${c.name}${c.required ? "" : "?"}: ${
                  c.control === "boolean"
                    ? "boolean"
                    : c.control === "number"
                      ? "number"
                      : c.control === "select"
                        ? c.options.map((o) => `"${String(o.value)}"`).join(" | ")
                        : "string"
                };`,
            )
            .join("\n")}\n}\n`;
        }
      } catch {
        // Ignore errors reading component file
      }
    }

    sendJson(palette);
  } catch (e) {
    sendError(e instanceof Error ? e.message : String(e));
  }
}
