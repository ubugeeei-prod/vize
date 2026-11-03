/**
 * SVG plot of the scaling lane: wall, CPU and peak RSS against SFC count per
 * tool (top row), and the same per SFC (bottom row), where a linear tool is
 * a flat line. No plotting dependency: plain SVG with a hover title on every
 * point, light and dark palettes, dashed lines for the non-Vize references.
 */

import { type Row, type ScalingResults, rowOf } from "./typecheck-scaling-report.ts";

/** Categorical slots 1-5 of the dataviz reference palette, light and dark. */
const SERIES = [
  ["#2a78d6", "#3987e5"],
  ["#eb6834", "#d95926"],
  ["#1baf7a", "#199e70"],
  ["#eda100", "#c98500"],
  ["#e87ba4", "#d55181"],
] as const;

type Panel = { title: string; unit: string; value: (row: Row) => number | null };
const PANELS: Panel[] = [
  { title: "Wall", unit: "s", value: (row) => (row.wallMs == null ? null : row.wallMs / 1000) },
  {
    title: "CPU (process tree)",
    unit: "s",
    value: (row) => (row.cpuMs == null ? null : row.cpuMs / 1000),
  },
  {
    title: "Peak RSS",
    unit: "GiB",
    value: (row) => (row.rssKb == null ? null : row.rssKb / 1024 ** 2),
  },
  {
    title: "Wall per SFC",
    unit: "ms",
    value: (row) => (row.wallMs == null ? null : row.wallMs / row.cell.sfcs),
  },
  {
    title: "CPU per SFC",
    unit: "ms",
    value: (row) => (row.cpuMs == null ? null : row.cpuMs / row.cell.sfcs),
  },
  {
    title: "Peak RSS per SFC",
    unit: "KiB",
    value: (row) => (row.rssKb == null ? null : row.rssKb / row.cell.sfcs),
  },
];

const PANEL_W = 300;
const PANEL_H = 220;
const GAP = 24;
const TOP = 72;
const PAD = { left: 52, right: 12, top: 28, bottom: 36 };

const escape = (text: string) =>
  text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

/** 1, 2 or 5 × 10^k at or above `max`, for an axis top with round ticks. */
export function niceCeiling(max: number): number {
  if (!(max > 0)) return 1;
  const power = 10 ** Math.floor(Math.log10(max));
  for (const step of [1, 2, 2.5, 5, 10]) if (step * power >= max) return step * power;
  return 10 * power;
}

const label = (value: number) =>
  value >= 100
    ? value.toFixed(0)
    : value >= 10
      ? value.toFixed(1)
      : value >= 1
        ? value.toFixed(2)
        : value.toPrecision(2);

export function renderScalingSvg(results: ScalingResults): string {
  const rows = results.cells.filter((cell) => cell.kind === "project").map(rowOf);
  const tools = [...new Map(rows.map((row) => [row.cell.tool, row.cell])).values()];
  const sizes = [...new Set(rows.map((row) => row.cell.sfcs))].sort((a, b) => a - b);
  const width = GAP + 3 * (PANEL_W + GAP);
  const height = TOP + 2 * (PANEL_H + GAP);
  const out: string[] = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" role="img" aria-labelledby="title">`,
    `<title id="title">Type-check cost against project size, ${escape(results.sha.slice(0, 12))}</title>`,
    "<style>",
    "svg{--surface:#fcfcfb;--ink:#0b0b0b;--muted:#52514e;--grid:#e4e3df;font-family:system-ui,sans-serif;font-size:11px}",
    ...tools.map((_, index) => `svg{--s${index}:${SERIES[index % SERIES.length]?.[0]}}`),
    "@media (prefers-color-scheme: dark){svg{--surface:#1a1a19;--ink:#f0efec;--muted:#c3c2b7;--grid:#3a3936}",
    ...tools.map((_, index) => `svg{--s${index}:${SERIES[index % SERIES.length]?.[1]}}`),
    "}",
    "text{fill:var(--ink)}.muted{fill:var(--muted)}.grid{stroke:var(--grid);stroke-width:1}",
    ".line{fill:none;stroke-width:2;stroke-linejoin:round}.dot{stroke:var(--surface);stroke-width:2}",
    "</style>",
    `<rect width="${width}" height="${height}" fill="var(--surface)"/>`,
    `<text x="${GAP}" y="24" font-size="15" font-weight="600">Type-check cost against project size</text>`,
    `<text x="${GAP}" y="42" class="muted">${escape(`${results.sha.slice(0, 12)} · ${results.runner} · median of ${results.runs} warm runs · flat bottom-row lines are linear`)}</text>`,
  ];
  let legendX = GAP;
  tools.forEach((cell, index) => {
    const dash = cell.vize ? "" : ' stroke-dasharray="5 3"';
    out.push(
      `<line x1="${legendX}" y1="58" x2="${legendX + 22}" y2="58" class="line" stroke="var(--s${index})"${dash}/>`,
      `<circle cx="${legendX + 11}" cy="58" r="4" class="dot" fill="var(--s${index})"/>`,
      `<text x="${legendX + 28}" y="62">${escape(cell.label)}</text>`,
    );
    legendX += 40 + cell.label.length * 6.2;
  });
  PANELS.forEach((panel, panelIndex) => {
    const x0 = GAP + (panelIndex % 3) * (PANEL_W + GAP);
    const y0 = TOP + Math.floor(panelIndex / 3) * (PANEL_H + GAP);
    const plotW = PANEL_W - PAD.left - PAD.right;
    const plotH = PANEL_H - PAD.top - PAD.bottom;
    const values = rows.map(panel.value).filter((value): value is number => value != null);
    const top = niceCeiling(Math.max(0, ...values));
    const maxSize = Math.max(1, ...sizes);
    const px = (sfcs: number) => x0 + PAD.left + (sfcs / maxSize) * plotW;
    const py = (value: number) => y0 + PAD.top + plotH - (value / top) * plotH;
    out.push(
      `<text x="${x0}" y="${y0 + 14}" font-weight="600">${escape(panel.title)} <tspan class="muted" font-weight="400">(${panel.unit})</tspan></text>`,
    );
    for (let tick = 0; tick <= 4; tick++) {
      const value = (top * tick) / 4;
      const y = py(value);
      out.push(
        `<line x1="${x0 + PAD.left}" y1="${y}" x2="${x0 + PAD.left + plotW}" y2="${y}" class="grid"/>`,
        `<text x="${x0 + PAD.left - 6}" y="${y + 4}" text-anchor="end" class="muted">${label(value)}</text>`,
      );
    }
    for (const sfcs of sizes) {
      out.push(
        `<text x="${px(sfcs)}" y="${y0 + PANEL_H - PAD.bottom + 16}" text-anchor="middle" class="muted">${sfcs >= 1000 ? `${sfcs / 1000}k` : sfcs}</text>`,
      );
    }
    out.push(
      `<text x="${x0 + PAD.left + plotW / 2}" y="${y0 + PANEL_H - 4}" text-anchor="middle" class="muted">SFCs</text>`,
    );
    tools.forEach((cell, index) => {
      const points = rows
        .filter((row) => row.cell.tool === cell.tool)
        .sort((a, b) => a.cell.sfcs - b.cell.sfcs)
        .flatMap((row) => {
          const value = panel.value(row);
          return value == null ? [] : [{ sfcs: row.cell.sfcs, value }];
        });
      if (points.length === 0) return;
      const dash = cell.vize ? "" : ' stroke-dasharray="5 3"';
      out.push(
        `<polyline class="line" stroke="var(--s${index})"${dash} points="${points.map((p) => `${px(p.sfcs).toFixed(1)},${py(p.value).toFixed(1)}`).join(" ")}"/>`,
      );
      for (const point of points) {
        out.push(
          `<circle cx="${px(point.sfcs).toFixed(1)}" cy="${py(point.value).toFixed(1)}" r="4" class="dot" fill="var(--s${index})"><title>${escape(`${cell.label}, ${point.sfcs} SFCs: ${label(point.value)} ${panel.unit}`)}</title></circle>`,
        );
      }
    });
  });
  out.push("</svg>");
  return `${out.join("\n")}\n`;
}
