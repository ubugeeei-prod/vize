# Retire MoonBit from repository tooling

Issue: [#8840](https://github.com/ubugeeei-prod/vize/issues/8840).

The maintainer decided on 2026-10-14 to remove MoonBit from the repository
tooling once the release commands were ported to TypeScript (#8844).

- The release driver always runs the prepared source's TypeScript release
  command. A source cut before the TypeScript port is refused instead of
  being prepared with the MoonBit release package, so **tags from before the
  port can no longer be re-released**.
- The release workflows drop their MoonBit publisher fallbacks and their
  `setup-moonbit` steps, and the operator no longer runs `moon update`.
- `tools/moon/`, its warning test, the MoonBit registry wrapper and the unused
  `moonScript` task helpers are removed; the retirement ledger keeps an empty
  MoonBit list so new `.mbt` tooling files are still rejected.

Scope: this retires MoonBit as a **tooling** language only. The MoonBit
template dialect (`davinci/vize_dialect_moonbit`), its `level-moonbit.yml`
lane and the `setup-moonbit` action that lane uses are product features
(see #8921) and stay.

Not yet done: the tooling workflows that still install MoonBit through
`setup-moonbit` for jobs that no longer run it, and the `tests/tooling`
helper still named `moonbit.ts`; both follow in a separate change.
