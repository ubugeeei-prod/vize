import type { CrossRuleMetadata, RuleProducer } from "./types.ts";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
const prefix = "vize:croquis/cf/";
export function crossMetadata(root: string): CrossRuleMetadata[] {
  const directory = resolve(root, "crates/vize_croquis_cf/src");
  const codes = [
    ...readFileSync(resolve(directory, "diagnostics/codes.rs"), "utf8").matchAll(
      /"(vize:croquis\/cf\/[a-z-]+)"/g,
    ),
  ].map((m) => m[1]);
  const variants = new Map(
    [
      ...readFileSync(resolve(directory, "diagnostics/rules.rs"), "utf8").matchAll(
        /CrossFileDiagnosticKind::(\w+)[\s\S]*?"(vize:croquis\/cf\/[a-z-]+)"/g,
      ),
    ].map((m) => [m[1], m[2]]),
  );
  const sources = new Map<string, RuleProducer[] | undefined>();
  function walk(path: string): void {
    for (const entry of readdirSync(resolve(root, path), { withFileTypes: true })) {
      const file = `${path}/${entry.name}`;
      if (entry.name.includes("test")) continue;
      if (entry.isDirectory()) walk(file);
      else if (entry.name.endsWith(".rs")) {
        const source = readFileSync(resolve(root, file), "utf8").split("#[cfg(test)]")[0];
        for (const [variant, code] of variants) {
          const at = source.indexOf(`CrossFileDiagnosticKind::${variant}`);
          if (at < 0) continue;
          const severity = source
            .slice(at, at + 1800)
            .match(/DiagnosticSeverity::(Error|Warning|Info|Hint)/)?.[1]
            ?.toLowerCase();
          const rows = sources.get(code) ?? [];
          rows.push({ path: file, severity });
          sources.set(code, rows);
        }
      }
    }
  }
  walk("crates/vize_croquis_cf/src/rules");
  sources.set(`${prefix}inject-without-symbol`, sources.get(`${prefix}provide-without-symbol`));
  return codes.map((code) => {
    const name = code.slice(prefix.length);
    const producer = sources.get(code) ?? [];
    const library = [
      "async-no-suspense",
      "inherit-attrs-unused",
      "multi-root-attrs",
      "unused-attrs",
      "undeclared-emit",
      "unused-emit",
      "unmatched-listener",
      "unhandled-event",
      "event-modifier",
      "missing-required-prop",
      "prop-type-mismatch",
      "undeclared-prop",
      "unregistered-component",
      "unresolved-import",
      "lifecycle-without-cleanup",
      "setup-context-violation",
    ].includes(name);
    const severities = [...new Set(producer.map((row) => row.severity).filter(Boolean))];
    return {
      code,
      name,
      producer,
      status: producer.length ? (library ? "library" : "cli") : "contract",
      severity:
        name === "unmatched-inject"
          ? "error / warning (with default)"
          : name === "setup-context-violation"
            ? "context-dependent"
            : severities.length
              ? severities.join(" / ")
              : "context-dependent",
    };
  });
}
export function explanation(root: string, code: string, ja: boolean) {
  const text = readFileSync(
    resolve(root, `crates/vize/src/commands/explain/snapshots/${ja ? "ja" : "en"}.txt`),
    "utf8",
  );
  const section = text.split(`=== ${code}\n`)[1]?.split("\n=== ")[0];
  if (!section) throw new Error(`Missing public explanation: ${code}`);
  const lines = section.split("\n");
  const help = lines.findIndex((line) => line === "help:" || line === "ヒント:");
  return {
    purpose: lines.slice(1, help).join("\n").trim(),
    help: lines
      .slice(help + 1)
      .map((line) => line.trim())
      .join(" ")
      .trim(),
  };
}
