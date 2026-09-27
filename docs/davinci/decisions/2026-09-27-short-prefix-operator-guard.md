# Short prefix-operator scan (#6868)

This private candidate starts at exact
`444ff3460aec443bbe9e83f700e3c142d5e7c1f0`. It changes only the prefix-operator
predicate: text of at most 31 UTF-8 bytes cannot contain a prefix run greater
than the existing limit of 31, so that predicate returns false immediately.
The full nesting analysis and OXC admission rule are unchanged.

## Consumed-byte proof

Every increment in `has_excessive_prefix_operator_run` can be charged to a
different existing source byte:

- `+`, `-`, `!` and `~` add one for one consumed byte.
- `++` and `--` add at most two for two bytes. The second byte is checked before
  either increment; an early refusal during the second increment still has
  that byte available to charge.
- `await`, `delete`, `typeof` and `void` add one after consuming the complete
  ASCII word, which is at least four bytes long.
- Resets reduce the run; every other branch adds nothing.

The outer cursor never goes backwards. Identifier/number helpers advance from
the byte after their opener. Quote helpers either stop at or after their input
cursor or advance past an escape. Comment helpers stop after the comment opener
or later. Successful regex skipping advances past its closer and optional
flags; unsuccessful regex skipping leaves the outer scan to consume its slash
once, and the speculative scan adds no operator increments. Template skipping
advances beyond a backtick opener or interpolation-closing brace, including any
following `${` opener. Interpolation bodies are scanned once in increasing byte
order; returning to literal text resets the operator run.

Thus no source byte contributes twice, even for malformed literals or skipped
text. A live run cannot exceed the input byte length. For length at most 31 the
original predicate always returns false, making the new early return equivalent.
The existing fallback body is preserved byte-exactly for longer input. No depth
limit, numeric-token limit, source cap or acceptance rule changes.

## Controls and evidence

The new disjoint `operators_tests.rs` module checks whole predicate and full
safety outcomes: 31 versus 32 unary operators, operators followed by an operand,
paired operators, all four prefix keywords, a live run spanning comments,
operators inside skipped literals, template interpolation runs and separate
runs. Short malformed delimiter/template/Unicode/escaped inputs still fail the
unchanged balancing guard, and an oversized numeric token remains refused.
The four existing ASCII admission/safety controls are untouched.

Local validation is limited to existing formatting, source-length and assertion
tools plus exact source comparisons. Rust tests are authored but **NOT RUN**.
Performance is **UNKNOWN**: shorter scans remove real producer work, but no
instruction saving, allocation result, cap compliance or runtime acceptance is
claimed. Compiler inlining and initial stack-query maps cost remain possible
effects of any source change.

The root owner must review this exact private delta before composing it with
other changes. Runtime correctness requires the existing hostile-input and
small-thread-stack controls; numeric acceptance requires the unchanged Actions
measurement of all 100 probes across three executions and the existing caps.
No build, install, push, PR, workflow dispatch, benchmark input/window change,
ceiling adjustment or stack-safety bypass was performed here. The roadmap owner
coordinates the final issue comment and central decision-record link when the
reviewed composition is ready; this companion is private source evidence.
