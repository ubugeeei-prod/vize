# Formatter queue instruction repair

Issue: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847).
Paired decision: [bounded production optimization](https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-5975577125).

The original provider candidate `f60de62eb632cbbca1b73e89244209adb31a0346`,
[Check 37168386441](https://github.com/ubugeeei-prod/vize/actions/runs/37168386441),
failed all four fixed formatter ceilings. It was removed from the queue.
Its complete original failure and receipts remain historical.

| Probe | Candidate instructions | Unchanged ceiling |
| --- | ---: | ---: |
| SFC | 293896 | 293575 |
| Reused SFC allocator | 275027 | 274723 |
| Large script | 929114 | 929044 |
| Complex template | 244583 | 244347 |

Read-only raw comparisons with accepted #7673 formatter artifact `11289310675`
find identical per-function exclusive counts in all three repetitions of each
source. The dependent #7682 packet gives the same counts despite a different
binary hash. Inputs, methodology, compiler, environment, harness and old public
formatter source remain unchanged. Added linked native source changes ThinLTO
code shape: actual optional sorting destruction becomes an out-of-line call,
with additional CSS/helper costs. The extra sorting cleanup belongs to the
formatter context; it is not an unused partial-default constructor.

The correction reduces actual production work while keeping existing behavior:

- The Unicode whitespace-only predicate stops at its first nonwhite scalar.
- No sorting override keeps pinned OXC's existing default `None`, avoiding an
  unnecessary assignment and drop. An actual override still clones its complete
  validated options.
- The existing thread-local expression scratch retains its six-byte ASCII
  `void (` prefix, truncating before appending the next original expression.
  The source type, actual parse, arena reset and printed extraction stay exact.
- CSS hash scanning already proves all bytes are ASCII hex digits. Admitting
  only lengths 3/4/6/8 directly replaces a redundant token parse; other color
  families still use the same CSS parser.

New controls preserve Unicode whitespace results, complete expression outputs
after long/rejected/short scratch reuse, and hash admission agreement with the
unchanged CSS parser across all tested lengths and letter cases. Original
source/options/output/error contracts, existing parser/formatter/stabilization
passes, native observation/admission and every probe/ceiling remain unchanged.

The provider keeps its original native runtime and laws. The dependent whole
SFC source must replay onto the final repaired provider and receive fresh source
Actions. A fresh three-run instruction campaign and exact-head PR checks must
qualify this repair before native Stack #7694 re-enters the protected queue.
Each protected candidate and literal signed merge remains a separate gate.
Historical passes do not accept this source; no gate waiver, profile or cap
change, extra pipeline stage, individual auto-merge, native default replacement
or #6882 closure is claimed.
