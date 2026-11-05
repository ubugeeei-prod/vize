import type { LegacyFixReport, Product } from "./report-legacy-fix-fixtures.ts";

/** PRs opened before this date stay report-only. */
export const GATE_SINCE = "2026-10-20T06:00:00Z";

/** Whether the requirement applies to a PR opened at `createdAt`; unknown means yes. */
export function enforced(createdAt: string | undefined): boolean {
  return !createdAt || Date.parse(createdAt) >= Date.parse(GATE_SINCE);
}

export function failingProducts(report: LegacyFixReport): Product[] {
  return report.products
    .filter((product) => product.state !== "new-inputs-registered")
    .map((product) => product.product);
}

export function formatSummary(report: LegacyFixReport, enforce: boolean): string {
  const lines = [
    "### Legacy fix differential inputs (#6852)",
    "",
    enforce
      ? "Required: each legacy product this fix changes must register a new active case with pinned inputs. Native parity is not checked here."
      : `Report only: this PR was opened before the requirement started (${GATE_SINCE}). Native parity is not checked here.`,
    "",
  ];
  if (report.products.length === 0) {
    lines.push(`Result: ${report.state}.`);
  } else {
    lines.push("| Product | Registered new input | Case IDs |", "| --- | --- | --- |");
    for (const product of report.products) {
      lines.push(`| ${product.product} | ${product.state} | ${product.cases.join(", ") || "—"} |`);
      if (product.reason) lines.push(`\n${product.product}: ${product.reason}`);
    }
    if (enforce && failingProducts(report).length > 0) {
      lines.push(
        "",
        "Add the fixed input under `tests/_fixtures/differential/<product>/<case>/` and register it at its sorted position in that product's `manifest.json` with its SHA-256.",
      );
    }
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}
