# Contextual prop anchor identity (#10661)

The original-from source is byte-identical to public repro commit
8195df603650e88e61381f4878bd8ac89ed1953a. Public vize 0.569.0 and the
published TypeScript 7.1 beta report unmapped TS2552 while the native CLI
incorrectly appears clean. The raw materialized native program independently
reports the same TS2552 with both native providers.

The keyword corpus covers all 84 keyword spellings from the pinned TypeScript
6.0.3 lexical token table. Local/import/prop shadowing, reactive destructures,
const-enum import type queries and Unicode identifiers retain lexical identity.
The genuine authored localNmae error must remain TS2552 at authored line3/column6.

All eight batch cases compare every returned production diagnostic. Because the
original native mapping drops the synthetic error, batch-clean alone is not
regression evidence: the separate required real-provider control checks complete
raw native programs and actual 7.1 beta mapper output over LF/CRLF/Unicode cases.
No diagnostic filter, source map, identifier keyword table or oracle changes.
