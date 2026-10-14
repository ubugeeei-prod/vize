import { stripVTControlCharacters } from "node:util";

type ProcessResult = { stdout: string; stderr: string; status: number | null };

/** Explicit untimed syntax and JS JSX fixtures require semantic recovery. */
export function normalizeParserRecovery<T extends ProcessResult>(
  result: T,
  expected: boolean | "js-jsx",
  phase: string,
): T {
  if (!expected || !["parity", "parity-profile"].includes(phase) || result.status !== 1)
    return result;
  const stderr = stripVTControlCharacters(result.stderr).trim();
  // The notice now names the first line of the failure; base binaries built
  // before that print only the cause class, so both spellings are accepted.
  const notice =
    /^warning: corsa: (?:Corsa CLI fast path unavailable; degraded to the slower project-session API|sharded Corsa CLI check failed; degraded to a single CLI program) \(check failure(?:: [^\n]*)?\)\. Type checking continues on a slower path\.(?:\nerror handling method 'exit': EOF|\ncontext canceled)?$/u;
  if (!notice.test(stderr)) return result;
  const report = JSON.parse(result.stdout) as {
    files?: { file?: string; diagnostics?: unknown[] }[];
  };
  if (
    !report.files?.some((file) =>
      file.diagnostics?.some(
        (diagnostic) =>
          typeof diagnostic === "string" &&
          (expected === "js-jsx"
            ? file.file === "loader.js" && /^error:\d+:\d+ \[TS7026\]/u.test(diagnostic)
            : /^error:\d+:\d+ \[TS1[0-4]\d{2}\]/u.test(diagnostic)),
      ),
    )
  )
    return result;
  // The caller has already retained raw stderr. Full report normalization and
  // exact base/head, one/two-server parity remain mandatory after this adapter.
  return { ...result, stderr: "" };
}
