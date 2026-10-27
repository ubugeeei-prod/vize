# Recovery source-copy cost

Recovery first parses the authored script. Valid input returns that parse.
For input with diagnostics, a repaired analysis buffer is needed only after a
recoverable member or call operator has been identified. Returning the original
parse without a repair must not first clone the entire script.

The production change delays that single buffer construction. Parser calls,
repair searches, replacement bytes, retry budgets and the complete returned
parser packet stay unchanged.

## Acceptance

The existing 50 complete ParserReturn comparisons remain byte-for-byte intact.
Three additional generated scripts contain actual TypeScript declarations,
Unicode identifiers and literals, and CRLF at at least 128 KiB, 1 MiB and 10 MiB.
An unrelated fatal error at the beginning must keep its original complete
parser packet, diagnostics and authored source.

An irreparable fatal parse retains its existing empty semantic result. Its
original ParserReturn, including syntax diagnostics and authored bytes, is
compared independently rather than interpreting empty bindings as accuracy.

The shared carrier has irreparable, valid and repairable cases. It constructs
source before the measured window and validates binding counts afterward.
Normal binaries measure repeated parse and analysis wall time. Separate
instrumented binaries export allocation requests and selected Croquis source
counts. Neither result is a cold whole-project benchmark.

The 4 KiB carrier must pass before the larger matrix runs:

- Both roles parse the original exactly once per repetition and report equal
  selected Croquis source calls and bytes. Irreparable input enters no semantic\n  analysis; the other cases enter it once per repetition.
- Irreparable input has no recovery parse. The candidate removes exactly one
  successful allocation request per repetition. Raw requested layout bytes are
  recorded; they are not physical copied bytes, live memory or RSS.
- Valid and repairable controls retain exactly equal allocation calls and
  requested bytes. Repairable input still has exactly one recovery parse.

The matrix repeats those laws for all three cases and all three source sizes.
A separate official Linux job qualifies instrumented allocation and source-count
laws first. It retains equal compiler flags and separate targets in both roles;
its elapsed times remain unqualified. Normal timing stays on the original
Blacksmith runner recipe and has separate source/binary provenance.
Normal timing uses three alternating baseline/candidate rounds. Whole-process
RSS receipts include generation and are kept separate from allocation windows.
A timing improvement is claimed only from completed normal measurements.

## Historical retry comparison

The original recovery carrier is specific to its incomplete-member fixture.
It admits only two counter classes:

- Two baseline repair parses to one candidate repair parse, with strictly fewer
  allocation calls and requested bytes.
- One repair parse in each role, with exactly equal original/recovery calls and
  source bytes and exactly equal allocation data.

Every candidate must retain exactly one repair parse. Missing or duplicate
counters, zero repetitions, unsupported counts and changes within the equal
class are rejected. Dedicated negative controls exercise these failures.
The original pinned two-to-one receipts are preserved as historical evidence.

Each role uses a separate target directory that includes the Actions run and
attempt IDs. The shared carrier, actual source roots, immutable first parent,
Cargo artifact manifests, features, profiles, lockfile and binary hashes are
retained. A small summary artifact holds separate tiny and matrix JSON; full raw
receipts remain available without uploading generated corpus files.

The TypeScript evidence readers preserve JSON integer tokens with BigInt.
Values beyond JavaScript's safe integer range are reported as exact decimal
strings; raw Rust receipts retain their original integer tokens.
