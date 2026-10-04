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
dependency. The lock adds only that existing dependency edge. The complete
original `parse_array_expression` body executes in
`stacker::maybe_grow(512 * 1024, 4 * 1024 * 1024, ...)`, before its token/span
work. It uses the same parser, arena, context, trailing-comma bookkeeping,
grammar and ordinary diagnostics. Flat scalar elements do not each gain a
stack check. No Vize dependency, depth limit, synthetic diagnostic, parse
retry, panic suppression, input skip or extra pipeline stage is added.

These constants reuse L0's stack-headroom configuration; that does not prove
parser frame sizes or ASan safety. Stack growth can allocate, and unsupported
stacker platforms do not grow. Computed members, TypeScript tuples/types,
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
using actual owned String conversions. One measured test-only inventory row
records alloc Vec `1/1` and L0 String `1/5`; all old rows remain byte-exact.
Parser source, original inputs, expected diagnostics and budgets are unchanged.
Actual nightly-ASan fixed-input replay is separately pending; no replay success
is claimed from these source laws. Root review precedes queue admission. The
protected full Rust suites and unchanged 104 instruction ceilings, actual
signed merge and literal reporter credit remain required. Legacy bytes,
historical laws, budgets and remaining Program/product completion gates stay
unchanged; #7805 remains open until actual replay and protected delivery.
