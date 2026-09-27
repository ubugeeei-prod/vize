# Expression safety dispatch hint (#6868)

This private candidate starts at exact
`56fcfd91a048c7e02693f12fb49f2f920b99920a`. The only production change is an
ordinary `#[inline]` hint on the existing public expression safety predicate.
It preserves the positive complete-ASCII-word path and the full fallback body.

## Evidence and decision

Actual run `36323473099` measures `armature_parse_medium` at 246053 versus
ceiling 243340. Independent raw attribution matches all three executions.
Its 29 retained-expression calls use 11245 inclusive Ir under the old predicate,
versus 13958 under the current extracted predicate: exactly +2713. The retained
parser's own work, OXC subtree (32985), trailing trivia and call counts are equal.

The old guard's self cost is 10117 with both scanners in its body. The current
cost separates into safety predicate 2273, nesting analysis 6116 and prefix
operator scan 4441: 12830 exclusive Ir in total, versus the old 10117. The
same shape affects other compound-expression probes. The ASCII branch also
adds work to failed classifications; this report does not attribute the entire
increase to call overhead alone.

The chosen hint targets this public dispatch boundary, which is called from
retained parsing, L2 admission and legacy transform/codegen. It allows the
compiler to reconsider that boundary in caller context without forcing every
scanner inline. It is ordinary `inline`, not `inline(always)`. Function placement,
scanners and semantic bodies stay fixed. Moving unrelated declarations to restore
a former file placement would lack a direct call-graph rationale and is not part
of this candidate.

This is a production optimization hypothesis, not a measured remedy. The current
100-probe source has multiple failed ceilings, and v-for's 147642 result has only
405 instructions of margin. The hint may leave scanners outlined, increase
code size or change other codegen decisions. No positive savings bound or gate
compliance follows from source inspection. Actual acceptance remains pending.

## Protected scope and checks

- Removing the single attribute restores `safety.rs` byte-exactly to frozen56fc.
  Its public path and complete body remain unchanged; the parent remains 350 lines.
- Existing numeric 4096/4097, depth/prefix, keyword, Unicode/escape and exact
  OXC source/AST/span controls remain byte-exact. All earlier complete v-for
  provenance controls and literals also remain unchanged. No mirrored attribute
  test or test expectation was added.
- Rustfmt, the actual assertion scanner and touched source350 checks pass.
  Protected raw comparison covers all 47 existing source/test/manifest/lock,
  benchmark/harness/driver, workflow, budget and guard paths.
- The 100 IDs, input digests, windows, protocol, ceilings and captured fixtures
  remain unchanged. No dependency, allocation gate, buffer code or stack-guard
  change is part of this candidate.

TODO: root reviews this concrete two-file patch before composition. The owner
records the paired issue/central decision with the final reviewed source, then
obtains unchanged full Rust/differential tests and actual 100×3 instruction
measurements. All protected ceilings must pass, including compound-expression
fallback probes and the existing v-for margin. No local Cargo/test build,
publication, dispatch or runtime acceptance was performed here.
