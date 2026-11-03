import { lexicalView } from "../../../tools/support/levels/move-host-runtime.ts";
import {
  hostCallers,
  hostImport,
} from "../../../tools/support/levels/move-timing-observer-host.ts";

/** Admit the single concrete observer import at its two existing host consumers. */
export function withoutTimingHostImport(source: string, file: string): string {
  if (!(hostCallers as readonly string[]).includes(file)) return source;
  const view = lexicalView(source);
  const matches = [...view.matchAll(/^use vize_carton::timing_observer::TimingObserver;$/gmu)];
  if (matches.length !== 1) return source;
  const match = matches[0];
  return (
    source.slice(0, match.index) +
    "host_timing_observer" +
    source.slice(match.index + hostImport.length)
  );
}
