# Original Program array-recursion stack headroom

Paired with [#7805](https://github.com/ubugeeei-prod/vize/issues/7805), the
actual `l1_program` crash from
[Fuzz run 37195977813](https://github.com/ubugeeei-prod/vize/actions/runs/37195977813)
at source `d544ed9a7d696d07a3f753df7c9e5edea2257d2f`.

## Authenticated original input

Artifact `11300504142`, `fuzz-reproducers-l1_program`, has ZIP SHA-256
`46d9af9b69b8cd939acb2abcf828dcfc6a0bf80364ec763a314b1cd3e683149f`.
Its member `crash-8b1a229cc533668f1cb8e773e57b25d5a68661f2` is 6,876 bytes,
SHA-256 `bd961cb21aa8b9541f088cb3ae29cbbab15700b2c5f3734adafbb07c37340c8d`.
The original selector is `3`: TypeScript, JSX enabled, Module goal.

The committed `tests/fuzz/regressions/l1_program/issue-7805-nested-array.tsx.input`
removes only that selector byte. Its 6,875 UTF-8 bytes have SHA-256
`e61f8019a498f6ba594723d56b4c7209e8310f602a3fcf9acd9051854232c6e0`,
5,596 opening brackets and no closing bracket. No trim, normalization,
minimization or source rewrite is performed. The existing seeder reads the
directory and prefixes every source with selectors `0..7`; its custody law
retains all eight historical hashes and checks all nine sources/72 seeds.
Reattaching selector `3` must reproduce the complete original member hash.

The authenticated failure is AddressSanitizer stack overflow in job
`111417810858`. The retained ASan frames are unsymbolized; they do not establish
named runtime frames. Source inspection identifies the recursive path from
array parsing through list elements, assignment and primary expressions back
to array parsing. Other recursion paths require their own evidence.

## Bounded parser correction

The existing pinned `stacker = "=0.1.25"` becomes a direct vendor-parser
dependency. The lock adds only that existing dependency edge. The complete original array parser body executes in
`stacker::maybe_grow(512 * 1024, 4 * 1024 * 1024, ...)`, before token/span work,
except exact adjacent `[]`, which cannot recurse and runs that same full body
without a stack query. The complete tuple body always retains its headroom guard. It uses the same parser, arena, context, trailing-comma bookkeeping,
grammar and ordinary diagnostics. Flat scalar elements do not each gain a
stack check. No Vize dependency, depth limit, synthetic diagnostic, parse
retry, panic suppression, input skip or extra pipeline stage is added.

These constants reuse L0's stack-headroom configuration; that does not prove
parser frame sizes or ASan safety. Stack growth can allocate, and unsupported
stacker platforms do not grow. Computed members, other TypeScript types,
parentheses, objects, JSX and later recursive AST consumers remain separate
paths. This change does not claim universal hostile-input safety or quotas.

## Source laws and unfinished acceptance

The L1 tests compare the exact original payload in all eight explicit profiles
on a 256 KiB thread and the ordinary test thread against ordinary parser
results on a 128 MiB reference thread. This reference uses the same patched
parser, not a historical parser measurement. Comparisons retain complete
source/profile, fatal/Flow/admission status, all owned diagnostic fields and
all comment fields. Unicode comment, spread, elision, trailing-comma and
malformed-array controls compare the same complete observations.

An 8,192-level valid array checks every actual arena node, authored node span
and numeric leaf iteratively in all eight small-thread profiles. It establishes
retained AST structure rather than accepting a new depth refusal. The ordinary
expression entry checks the same tree and genuine caller unwind after parsing;
it does not claim a panic injected while a grown segment remains active.
Owned results drop before their original allocator. No recursive AST Debug,
serialization or visitor is used for these checks.

Source compilation and execution remain pending until fresh automatic Actions.
Initial source `b6923eb8ea5ca83d99f9012d5f2d9cea2bdfe495`, Check
`37224517771`, stopped at authentic Rust Build job `111501400083` with seven
`E0433`/`E0425`/`E0599` errors: the new `no_std` unit module lacked explicit
test-only std/container imports. Its four laws were unexecuted. The correction
adds `extern crate std`, `alloc::vec::Vec` and preferred `vize_l0::String`,
using actual owned String conversions. Its measured test-only inventory row
initially recorded alloc Vec `1/1` and L0 String `1/5`.
Parser source, original inputs, expected diagnostics and budgets are unchanged.
That import repair at `d0114e1899b09493228dd605b0d9d58857e70c29`, Check
`37224992778`, compiled successfully. Worker `111503918025` passed 3,864 of
3,865 tests but the original eight-profile law still aborted with stack
overflow. The other three workers passed. This is a real failed acceptance;
the anonymous thread report does not establish its active profile or named
frames. Static inspection reveals another unguarded recursive route during
TypeScript `<` speculation: type arguments, indexed access, tuple parsing,
tuple element and `parse_ts_type`. The next correction wraps the complete
original tuple parser before its token work with the same headroom guard,
preserving every rest/optional diagnostic and final AST allocation. The
original full-payload comparison remains unchanged. A fifth source law checks
every actual node and span of an 8,192-level valid tuple in all four TS profiles
on small threads. The only inventory change adds its one owned L0 String use,
making `1/6`; old rows and all budgets stay unchanged. Fresh execution remains
pending, and the static route is not claimed as an authenticated runtime trace.
Actual nightly-ASan fixed-input replay is separately pending; no replay success
is claimed from these source laws. Root review precedes queue admission. The
protected full Rust suites and unchanged 104 instruction ceilings, actual
signed merge and literal reporter credit remain required. Legacy bytes,
historical laws, budgets and remaining Program/product completion gates stay
unchanged; #7805 remains open until actual replay and protected delivery.

The tuple correction at `092a19bc13a53d8925a2bdd9d278637d30c5ac29`
has terminal-success metadata for all nine jobs of existing replay
`37245166243`. Its normal PR Check did not start while its old `61c975f8`
base conflicted with later main. The three meaningful source commits were
therefore genuinely rebased onto actual Nuxt 2 delivery `5d879b2724735749a0d045f11fa5cfcd77542922`.
All ten owned non-record blobs remain byte-identical to `092a19bc`; every
incoming canonical and storage row is retained. Old replay and new-source
acceptance stay distinct; new automatic PR Actions and current-source replay
must qualify the refreshed head before protected admission.

## Measured formatter regression and nonrecursive empty literal

Current source `037f8bb657213bfddd74c5b5e561e0018478e7e8` passed automatic
Check 37246096668 and fresh all-nine-target Fuzz replay 37247328255. Raw
`l1_program` replay completes 945 runs over 944 seeds; expression replay retains
its 22,933-byte maximum and completes 337 runs. These source receipts remain
separate from protected delivery and final release gates.

Protected candidate `0640b734999ebce32dcb50e195a027575f2e927d`, Check
37247858665/job 111569230804, passed the original 100 stage probes but failed
`formatter_script_large`: all three original-window measurements were 929,093
against the unchanged 929,044 ceiling. It was immediately dequeued; no blind
retry, fixture/window change, environment change or ceiling increase followed.
First raw log SHA256 is
`4c2c584c6bcadf7849af3b36ac07cf69bf8b03e0821c385449c74cdb0b07d729`.
Small raw artifacts 11318924999 and 11319249963 remain preserved.

Independent reconstruction compares exact base 2c33e33a with candidate 0640b734.
The base repeats 928,913 and has identical per-function costs to preceding 85463bfe.
The +180 instructions reconcile exactly to new array closure +274, newly
outlined delimited-list +72, primary-expression -234, binding-pattern +70,
try-statement closure -32, remaining-stack +26 and stack-pointer +4. The
original full `format_script` measurement parses both the input and stabilized
output; each contains one authored `items: []`. Two direct stack queries occur
inside that original measurement; no stack growth/switch cost is observed.
This is a measured shallow-path regression, not timing noise.

Before any lexer mutation, the correction reads only the original source byte
at the current opening token's end. Only an adjacent `]` bypasses the stack
query. It still executes the complete original private body: opening span,
expectation, In context, delimited-list, comma state, closing expectation and
actual arena AST allocation. The original delimited-list returns before its
element callback for that closing token, so this case cannot recurse.
Whitespace, comments, nonempty arrays, malformed input and EOF retain the
headroom guard. The tuple guard, original crash input and all five earlier
source laws remain intact. A sixth law compares complete observations in all
eight profiles for adjacent, nested, whitespace/comment, elision and malformed
controls on small threads; it introduces no additional owned storage use.

Fresh source Actions, exact-head unchanged 100+4 instruction measurements,
current-head sanitizer replay and protected actual delivery remain required.
No saved count or successful source/queue/release result is inferred from the
static optimization. v0.433.0 remains unpublished; supported resume must refresh
from the final accepted literal main and rerun the effective release contract.
