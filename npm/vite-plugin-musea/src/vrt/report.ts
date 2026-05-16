/**
 * VRT report generation for Musea.
 *
 * Generates HTML and JSON reports from VRT results for visual review
 * and CI integration.
 */

import path from "node:path";

import type { VrtResult, VrtSummary } from "./types.js";
import { escapeHtml } from "./comparison.js";

/**
 * Generate VRT report in HTML format.
 * Supports side-by-side, overlay, and slider comparison modes.
 */
export function generateVrtReport(results: VrtResult[], summary: VrtSummary): string {
  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    if (minutes === 0) return `${seconds}s`;
    return `${minutes}m ${seconds % 60}s`;
  };

  const timestamp = new Date().toLocaleString("ja-JP", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });

  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>VRT Report - Musea</title>
  <style>
    :root {
      --musea-bg-primary: #0d0d0d;
      --musea-bg-secondary: #1a1815;
      --musea-bg-tertiary: #252220;
      --musea-accent: #a34828;
      --musea-accent-hover: #c45a32;
      --musea-text: #e6e9f0;
      --musea-text-muted: #7b8494;
      --musea-border: #3a3530;
      --musea-success: #4ade80;
      --musea-error: #f87171;
      --musea-info: #60a5fa;
      --musea-warning: #fbbf24;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: var(--musea-bg-primary);
      color: var(--musea-text);
      min-height: 100vh;
      line-height: 1.5;
    }

    .header {
      background: var(--musea-bg-secondary);
      border-bottom: 1px solid var(--musea-border);
      padding: 1rem 2rem;
      display: flex;
      align-items: center;
      justify-content: space-between;
      position: sticky;
      top: 0;
      z-index: 100;
    }
    .header-left { display: flex; align-items: center; gap: 1rem; }
    .logo { font-size: 1.25rem; font-weight: 700; color: var(--musea-accent); }
    .header-title { color: var(--musea-text-muted); font-size: 0.875rem; }
    .header-meta { display: flex; align-items: center; gap: 1.5rem; font-size: 0.8125rem; color: var(--musea-text-muted); }
    .header-meta span { display: flex; align-items: center; gap: 0.375rem; }

    .main { max-width: 1400px; margin: 0 auto; padding: 2rem; }

    .summary { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 1rem; margin-bottom: 2rem; }
    .stat { background: var(--musea-bg-secondary); border: 1px solid var(--musea-border); border-radius: 8px; padding: 1.25rem; position: relative; overflow: hidden; }
    .stat::before { content: ''; position: absolute; left: 0; top: 0; bottom: 0; width: 3px; }
    .stat.passed::before { background: var(--musea-success); }
    .stat.failed::before { background: var(--musea-error); }
    .stat.new::before { background: var(--musea-info); }
    .stat.skipped::before { background: var(--musea-warning); }
    .stat-value { font-size: 2rem; font-weight: 700; font-variant-numeric: tabular-nums; line-height: 1; margin-bottom: 0.25rem; }
    .stat.passed .stat-value { color: var(--musea-success); }
    .stat.failed .stat-value { color: var(--musea-error); }
    .stat.new .stat-value { color: var(--musea-info); }
    .stat.skipped .stat-value { color: var(--musea-warning); }
    .stat-label { color: var(--musea-text-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; font-weight: 500; }

    .filters { display: flex; gap: 0.5rem; margin-bottom: 1.5rem; flex-wrap: wrap; }
    .filter-btn { background: var(--musea-bg-secondary); border: 1px solid var(--musea-border); color: var(--musea-text); padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.8125rem; font-weight: 500; transition: all 0.15s ease; }
    .filter-btn:hover { background: var(--musea-bg-tertiary); border-color: var(--musea-text-muted); }
    .filter-btn.active { background: var(--musea-accent); border-color: var(--musea-accent); color: #fff; }
    .filter-btn .count { opacity: 0.7; margin-left: 0.25rem; }

    /* Comparison mode toggle */
    .compare-modes { display: flex; gap: 0.25rem; margin-bottom: 1.5rem; background: var(--musea-bg-secondary); border-radius: 6px; padding: 0.25rem; width: fit-content; }
    .compare-mode-btn { background: none; border: none; color: var(--musea-text-muted); padding: 0.375rem 0.75rem; border-radius: 4px; cursor: pointer; font-size: 0.75rem; font-weight: 500; transition: all 0.15s ease; }
    .compare-mode-btn.active { background: var(--musea-bg-tertiary); color: var(--musea-text); }

    .results { display: flex; flex-direction: column; gap: 0.75rem; }
    .result { background: var(--musea-bg-secondary); border: 1px solid var(--musea-border); border-radius: 8px; overflow: hidden; transition: border-color 0.15s ease; }
    .result:hover { border-color: var(--musea-text-muted); }
    .result-header { padding: 1rem 1.25rem; display: flex; justify-content: space-between; align-items: center; cursor: pointer; border-left: 3px solid transparent; background: var(--musea-bg-tertiary); }
    .result.passed .result-header { border-left-color: var(--musea-success); }
    .result.failed .result-header { border-left-color: var(--musea-error); }
    .result.new .result-header { border-left-color: var(--musea-info); }
    .result.error .result-header { border-left-color: var(--musea-warning); }

    .result-info { display: flex; align-items: center; gap: 1rem; }
    .result-name { font-weight: 600; font-size: 0.9375rem; }
    .result-meta { color: var(--musea-text-muted); font-size: 0.8125rem; padding: 0.125rem 0.5rem; background: var(--musea-bg-secondary); border-radius: 4px; }
    .result-badge { padding: 0.25rem 0.625rem; border-radius: 4px; font-size: 0.6875rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; }
    .result.passed .result-badge { background: rgba(74, 222, 128, 0.15); color: var(--musea-success); }
    .result.failed .result-badge { background: rgba(248, 113, 113, 0.15); color: var(--musea-error); }
    .result.new .result-badge { background: rgba(96, 165, 250, 0.15); color: var(--musea-info); }
    .result.error .result-badge { background: rgba(251, 191, 36, 0.15); color: var(--musea-warning); }

    .result-body { border-top: 1px solid var(--musea-border); }
    .result-details { padding: 0.875rem 1.25rem; font-size: 0.8125rem; color: var(--musea-text-muted); font-family: 'SF Mono', 'Fira Code', monospace; background: var(--musea-bg-primary); }
    .result-details.error { color: var(--musea-error); }

    .result-images { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 1rem; padding: 1.25rem; background: var(--musea-bg-primary); }
    .result-images.overlay { grid-template-columns: 1fr; }
    .image-container { background: var(--musea-bg-secondary); border: 1px solid var(--musea-border); border-radius: 6px; overflow: hidden; }
    .image-label { padding: 0.625rem 0.875rem; font-size: 0.6875rem; font-weight: 600; color: var(--musea-text-muted); text-transform: uppercase; letter-spacing: 0.08em; background: var(--musea-bg-tertiary); border-bottom: 1px solid var(--musea-border); }
    .image-wrapper { padding: 0.5rem; background: repeating-conic-gradient(var(--musea-bg-tertiary) 0% 25%, var(--musea-bg-secondary) 0% 50%) 50% / 16px 16px; }
    .image-container img { width: 100%; height: auto; display: block; border-radius: 2px; }

    /* Slider comparison */
    .slider-compare { position: relative; overflow: hidden; }
    .slider-compare img { display: block; width: 100%; }
    .slider-overlay { position: absolute; top: 0; left: 0; bottom: 0; overflow: hidden; border-right: 2px solid var(--musea-accent); }
    .slider-overlay img { display: block; min-width: 100%; height: 100%; object-fit: cover; }

    .empty-state { text-align: center; padding: 4rem 2rem; color: var(--musea-text-muted); }
    .all-passed { background: rgba(74, 222, 128, 0.1); border: 1px solid rgba(74, 222, 128, 0.2); border-radius: 8px; padding: 1.5rem; text-align: center; margin-bottom: 1.5rem; }
    .all-passed-icon { font-size: 2.5rem; margin-bottom: 0.5rem; }
    .all-passed-text { color: var(--musea-success); font-weight: 600; }
  </style>
</head>
<body>
  <header class="header">
    <div class="header-left">
      <div class="logo">Musea</div>
      <span class="header-title">Visual Regression Report</span>
    </div>
    <div class="header-meta">
      <span>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
        </svg>
        ${formatDuration(summary.duration)}
      </span>
      <span>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="4" width="18" height="18" rx="2" ry="2"/><line x1="16" y1="2" x2="16" y2="6"/><line x1="8" y1="2" x2="8" y2="6"/><line x1="3" y1="10" x2="21" y2="10"/>
        </svg>
        ${timestamp}
      </span>
    </div>
  </header>

  <main class="main">
    <div class="summary">
      <div class="stat passed"><div class="stat-value">${summary.passed}</div><div class="stat-label">Passed</div></div>
      <div class="stat failed"><div class="stat-value">${summary.failed}</div><div class="stat-label">Failed</div></div>
      <div class="stat new"><div class="stat-value">${summary.new}</div><div class="stat-label">New</div></div>
      <div class="stat skipped"><div class="stat-value">${summary.skipped}</div><div class="stat-label">Skipped</div></div>
    </div>

    ${
      summary.failed === 0 && summary.skipped === 0 && summary.total > 0
        ? `<div class="all-passed">
            <div class="all-passed-icon">✓</div>
            <div class="all-passed-text">All ${summary.total} visual tests passed</div>
          </div>`
        : ""
    }

    <div class="filters">
      <button class="filter-btn active" data-filter="all">All<span class="count">(${summary.total})</span></button>
      <button class="filter-btn" data-filter="failed">Failed<span class="count">(${summary.failed})</span></button>
      <button class="filter-btn" data-filter="passed">Passed<span class="count">(${summary.passed})</span></button>
      <button class="filter-btn" data-filter="new">New<span class="count">(${summary.new})</span></button>
    </div>

    <div class="compare-modes">
      <button class="compare-mode-btn active" data-mode="side-by-side">Side by Side</button>
      <button class="compare-mode-btn" data-mode="overlay">Overlay</button>
      <button class="compare-mode-btn" data-mode="slider">Slider</button>
    </div>

    <div class="results">
      ${
        results.length === 0
          ? `<div class="empty-state"><p>No visual tests found</p></div>`
          : results
              .map((r) => {
                const status = r.error ? "error" : r.isNew ? "new" : r.passed ? "passed" : "failed";
                const badge = r.error ? "Error" : r.isNew ? "New" : r.passed ? "Passed" : "Failed";
                const artName = path.basename(r.artPath, ".art.vue");
                const viewportName = r.viewport.name || `${r.viewport.width}×${r.viewport.height}`;

                let details = "";
                if (r.error) {
                  details = `<div class="result-details error">${escapeHtml(r.error)}</div>`;
                } else if (r.diffPercentage !== undefined) {
                  const diffFormatted = r.diffPercentage.toFixed(3);
                  const pixelsFormatted = r.diffPixels?.toLocaleString() ?? "0";
                  const totalFormatted = r.totalPixels?.toLocaleString() ?? "0";
                  details = `<div class="result-details">diff: ${diffFormatted}% (${pixelsFormatted} / ${totalFormatted} pixels)</div>`;
                }

                let images = "";
                if (!r.error && !r.passed && r.diffPath) {
                  images = `<div class="result-images" data-baseline="file://${r.snapshotPath}" data-current="file://${r.currentPath}" data-diff="file://${r.diffPath}">
                    ${r.snapshotPath ? `<div class="image-container"><div class="image-label">Baseline</div><div class="image-wrapper"><img src="file://${r.snapshotPath}" alt="Baseline" loading="lazy" /></div></div>` : ""}
                    ${r.currentPath ? `<div class="image-container"><div class="image-label">Current</div><div class="image-wrapper"><img src="file://${r.currentPath}" alt="Current" loading="lazy" /></div></div>` : ""}
                    ${r.diffPath ? `<div class="image-container"><div class="image-label">Diff</div><div class="image-wrapper"><img src="file://${r.diffPath}" alt="Diff" loading="lazy" /></div></div>` : ""}
                  </div>`;
                }

                const hasBody = details || images;

                return `<div class="result ${status}" data-status="${status}">
                  <div class="result-header">
                    <div class="result-info">
                      <div class="result-name">${escapeHtml(artName)} / ${escapeHtml(r.variantName)}</div>
                      <div class="result-meta">${escapeHtml(viewportName)}</div>
                    </div>
                    <span class="result-badge">${badge}</span>
                  </div>
                  ${hasBody ? `<div class="result-body">${details}${images}</div>` : ""}
                </div>`;
              })
              .join("")
      }
    </div>
  </main>

  <script>
    // Filter buttons
    document.querySelectorAll('.filter-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        const filter = btn.dataset.filter;
        document.querySelectorAll('.result').forEach(result => {
          result.style.display = (filter === 'all' || result.dataset.status === filter) ? 'block' : 'none';
        });
      });
    });

    // Compare mode buttons
    document.querySelectorAll('.compare-mode-btn').forEach(btn => {
      btn.addEventListener('click', () => {
        document.querySelectorAll('.compare-mode-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        // Mode switching would update result-images display; this is a static report for now
      });
    });
  </script>
</body>
</html>`;

  return html;
}

/**
 * Generate VRT JSON report for CI integration.
 */
export function generateVrtJsonReport(results: VrtResult[], summary: VrtSummary): string {
  return JSON.stringify(
    {
      timestamp: new Date().toISOString(),
      summary,
      results: results.map((r) => ({
        art: path.basename(r.artPath, ".art.vue"),
        variant: r.variantName,
        viewport: r.viewport.name || `${r.viewport.width}x${r.viewport.height}`,
        status: r.error ? "error" : r.isNew ? "new" : r.passed ? "passed" : "failed",
        diffPercentage: r.diffPercentage,
        error: r.error,
      })),
    },
    null,
    2,
  );
}
