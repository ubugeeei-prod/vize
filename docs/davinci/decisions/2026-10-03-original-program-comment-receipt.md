# Original Program comment observation

Issues: [#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The genuine File provider privately retains whether the admitted original
Program's comment vector is nonempty. Its existing `ProgramOrigin::checked`
reads this fact in constant time before any observer callback; no source scan,
additional parse, traversal or pipeline stage is introduced. The immutable
`ScriptUnit::has_comments` getter reports the same original fact even after
callback interruption. It grants neither completed File nor filename authority.

Comments can contain filename-sensitive TypeScript reference directives or
JavaScript JSDoc imports without producing an import/export or invocation row.
Canon's bounded native whole-Program adapter must therefore refuse every
commented Program before spawning its backend until an actual provider owns
authored filename authority. The independent L4 projection still preserves and
maps comments; this conservative checker refusal does not restrict emission.

The source change is a true child of the original invocation provider. Tests
use actual stock-parser admission and the existing File walk once, covering
comment-free spelling, comment-like strings, line/block/JSDoc/reference comments
in JS and TS, and an original tail comment retained before unit callback unwind.
Actual storage layout is compared with the original provider using compiler
layout output. Exact-head Actions and protected merge acceptance remain required.
