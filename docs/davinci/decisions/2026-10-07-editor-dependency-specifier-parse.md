# Share the editor dependency module parse

Tracking decision: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-6020173789).

The editor surface walk parsed each dependency module separately to collect
relative Vue imports, relative script imports and alias imports. These collectors
visit the same module literals in the same order. The walk now collects the
existing ordered, deduplicated module list once and applies those three existing
classifications to it. This removes one parse when no aliases/package routes
exist, and two when they do. Each dependency is still visited and resolved.
Membership uses a local hash set while results append in their first AST order,
avoiding quadratic deduplication when alias-free modules have many bare imports.
That temporary set belongs only to this one collection and caches no authority.

Relative Vue probes retain explicit and extensionless spellings, deduplication
after mapping, and their existing order. Script classification retains exclusions
for `.vue`, `.vue.ts` and `.vue.tsx`; aliases retain importer/package ownership.
Static imports, exports, dynamic literal imports, CommonJS requires, TypeScript
import types, external module references and augmentations retain their original
coverage. Comments, unrelated strings and computed imports remain excluded.

The independent original relative-script AST collector remains test-only.
Whole-list differential controls cover JS, TS, JSX and TSX; Vue controls pin the
complete literal order and extensionless filesystem probes. Existing corpus
inputs, generated code, mappings, fallbacks, route/config/source stamps and all
invalidation obligations remain unchanged. No cross-request cache, worker,
readiness barrier, native query, pipeline stage or dependency is added.

The existing automatic original400 Actions pair admits only the five exact
production/test paths and the byte-exact moved original test oracle. Its original inputs, 534 ACKs,
128-frame limits, 16 workers, deadlines, epochs/generations, complete 79 answers
and 18 notifications stay intact. Timing and output qualification are pending;
source parse-count reduction does not establish a hover latency improvement or
the requested 10x full-command result. Publication remains owned by the parent
release lane after fresh source checks and protected actual merge.
