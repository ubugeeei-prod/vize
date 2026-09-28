# L1→L2 provenance profile keys (#6832)

`vize_l1_to_l2::lower::benchmark` owns the current provenance recording span
and retained-storage counters. Its profile export keys use the conversion
registry's `l1_to_l2` stage ID. This is a wire rename for the native lowering
run; counter meaning, span timing, profiler enablement, and the generic profile
export schema stay the same. The old keys have no native read alias.
The existing `davinci-benchmark-profile` feature still gates this optional
measurement; changing that selector belongs to a separate feature migration.
After a conflict, run `python3 tools/support/levels/rename-provenance-profile-keys.py --write`
on current `main`; `--check` requires all ten current keys and the replay is
idempotent and scoped to the one producer file.

| Old key | Current key |
| --- | --- |
| `davinci.lower.provenance.record` | `l1_to_l2.provenance.record` |
| `davinci.lower.provenance.records` | `l1_to_l2.provenance.records` |
| `davinci.lower.provenance.vector_capacity_bytes` | `l1_to_l2.provenance.vector_capacity_bytes` |
| `davinci.lower.provenance.record_bytes` | `l1_to_l2.provenance.record_bytes` |
| `davinci.lower.provenance.rule_heap_strings` | `l1_to_l2.provenance.rule_heap_strings` |
| `davinci.lower.provenance.rule_heap_capacity_bytes` | `l1_to_l2.provenance.rule_heap_capacity_bytes` |
| `davinci.lower.provenance.before_heap_strings` | `l1_to_l2.provenance.before_heap_strings` |
| `davinci.lower.provenance.before_heap_capacity_bytes` | `l1_to_l2.provenance.before_heap_capacity_bytes` |
| `davinci.lower.provenance.after_heap_strings` | `l1_to_l2.provenance.after_heap_strings` |
| `davinci.lower.provenance.after_heap_capacity_bytes` | `l1_to_l2.provenance.after_heap_capacity_bytes` |

The source-built integration test pins the one span, all nine counters,
their retained-record arithmetic, and disabled profiling. The three immutable
SFC fix-history observation JSON files, legacy product output, and historical
profile captures are not rewritten. Differential feature selectors were renamed
in #7074; other current profile and remark families still need separate
producer/consumer migrations.
