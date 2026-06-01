/**
 * Preview HTML page generation for Musea component previews.
 *
 * Generates the HTML wrapper pages that host Vue component previews
 * inside iframes.
 *
 * Extracted from preview.ts to keep file sizes manageable.
 */

import type { ArtFileInfo, ArtVariant } from "../types/index.js";
import { escapeHtml } from "../utils.js";

export function generatePreviewHtml(
  art: ArtFileInfo,
  variant: ArtVariant,
  _basePath: string,
  viteBase?: string,
): string {
  // Use preview-module HTTP endpoint instead of virtual module import.
  // Virtual module imports in inline scripts require transformIndexHtml,
  // which creates malformed html-proxy URLs when the page URL has query params.
  const previewModuleUrl = `${_basePath}/preview-module?art=${encodeURIComponent(art.path)}&variant=${encodeURIComponent(variant.name)}`;
  const base = (viteBase || "/").replace(/\/$/, "");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(art.metadata.title)} - ${escapeHtml(variant.name)}</title>
  <script type="module" src="${base}/@vite/client"></script>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    :root {
      --musea-preview-padding: clamp(1rem, 2vw, 1.5rem);
    }
    html, body {
      width: 100%;
      height: 100%;
    }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: #ffffff;
      padding: var(--musea-preview-padding);
      overflow: auto;
    }
    .musea-variant {
      width: 100%;
      min-height: calc(100vh - (var(--musea-preview-padding) * 2));
    }
    .musea-error {
      color: #dc2626;
      background: #fef2f2;
      border: 1px solid #fecaca;
      border-radius: 8px;
      padding: 1rem;
      font-size: 0.875rem;
      max-width: 400px;
    }
    .musea-error-title {
      font-weight: 600;
      margin-bottom: 0.5rem;
    }
    .musea-error pre {
      font-family: monospace;
      font-size: 0.75rem;
      white-space: pre-wrap;
      word-break: break-all;
      margin-top: 0.5rem;
      padding: 0.5rem;
      background: #fff;
      border-radius: 4px;
    }
    .musea-loading {
      display: flex;
      align-items: center;
      gap: 0.75rem;
      color: #6b7280;
      font-size: 0.875rem;
    }
    .musea-spinner {
      width: 20px;
      height: 20px;
      border: 2px solid #e5e7eb;
      border-top-color: #3b82f6;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* Musea Addons: Checkerboard background for transparent mode */
    .musea-bg-checkerboard {
      background-image:
        linear-gradient(45deg, #ccc 25%, transparent 25%),
        linear-gradient(-45deg, #ccc 25%, transparent 25%),
        linear-gradient(45deg, transparent 75%, #ccc 75%),
        linear-gradient(-45deg, transparent 75%, #ccc 75%) !important;
      background-size: 20px 20px !important;
      background-position: 0 0, 0 10px, 10px -10px, -10px 0 !important;
    }

    /* Musea Addons: Measure label */
    .musea-measure-label {
      position: fixed;
      background: #333;
      color: #fff;
      font-size: 11px;
      padding: 2px 6px;
      border-radius: 3px;
      pointer-events: none;
      z-index: 100000;
    }
  </style>
</head>
<body>
  <div id="app" class="musea-variant" data-art="${escapeHtml(art.path)}" data-variant="${escapeHtml(variant.name)}">
    <div class="musea-loading">
      <div class="musea-spinner"></div>
      Loading component...
    </div>
  </div>
  <script type="module" src="${previewModuleUrl}"></script>
</body>
</html>`;
}
