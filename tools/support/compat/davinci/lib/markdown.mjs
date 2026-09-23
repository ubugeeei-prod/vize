// Markdown emission helpers for the Davinci generated artifacts.
//
// `vp check` formats markdown tables to prettier's aligned style (cells
// padded to the column's maximum width; right-aligned columns pad on the
// left and use a `----:` separator). Generated artifacts are byte-compared
// by their staleness checks, so the generators must emit that canonical
// form directly - this helper is the single implementation of it.

/**
 * Render a markdown table in the vp-canonical aligned style.
 *
 * @param {string[]} headers - column headers
 * @param {("left"|"right")[]} aligns - per-column alignment
 * @param {string[][]} rows - cell contents (already stringified)
 * @returns {string} the table, one trailing newline, no leading newline
 */
export function formatTable(headers, aligns, rows) {
  const widths = headers.map((header, column) => {
    let width = header.length;
    for (const row of rows) {
      width = Math.max(width, String(row[column] ?? "").length);
    }
    // Prettier keeps separator cells at least three dashes wide.
    return Math.max(width, 3);
  });

  const renderCell = (content, column) => {
    const text = String(content ?? "");
    const pad = " ".repeat(widths[column] - text.length);
    return aligns[column] === "right" ? pad + text : text + pad;
  };

  const lines = [];
  lines.push(`| ${headers.map((header, i) => renderCell(header, i)).join(" | ")} |`);
  lines.push(
    `| ${widths
      .map((width, i) => (aligns[i] === "right" ? `${"-".repeat(width - 1)}:` : "-".repeat(width)))
      .join(" | ")} |`,
  );
  for (const row of rows) {
    lines.push(`| ${row.map((cell, i) => renderCell(cell, i)).join(" | ")} |`);
  }
  return `${lines.join("\n")}\n`;
}
