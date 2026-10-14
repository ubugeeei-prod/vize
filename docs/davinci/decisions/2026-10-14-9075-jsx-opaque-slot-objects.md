# JSX slot objects that Vize cannot expand

Issue [#9075](https://github.com/ubugeeei-prod/vize/issues/9075). `vize lint` reported
`parser/jsx` findings on valid Vue TSX, even under `preset: "incremental"`, and the
compiler dropped the slots behind them. Every finding in the corpus had one of
three shapes:

- A slots object entry that is a function reference, e.g.
  `v-slots={{ default: slots.icon }}` in vant, or a sole object child
  `{ default: createChildren }` in naive-ui. Vize expands only inline functions
  into slot templates, so it warned and dropped the entry.
- A spread in a slots object, e.g. `v-slots={{ ...slots, … }}` in vuetify. Vize
  warned and dropped the spread.
- `v-slots:default={slots.prepend}` in vuetify. Vize rejected the argument as an
  error.

## Decision

`@vue/babel-plugin-jsx` copies a slots object's entries as written, and it reads
any `v-slots:arg` as `v-slots` with the argument ignored. Vize now does the same
where it can reproduce the result exactly:

- A slots object with a spread or a non-function entry is forwarded verbatim when
  it contains no JSX and no TypeScript-only syntax. Its source is then exactly
  the object Babel emits. It uses the existing opaque-slots path from #3467, so
  Vapor and SSR keep reporting that they cannot express a forwarded object.
- `v-slots:arg={…}` is lowered like `v-slots={…}`. This reverses the stricter
  rule in the Babel compatibility inventory.

An object that mixes such entries with JSX is still expanded and still reports
the entries it drops. Forwarding it would leave JSX in the emitted JavaScript.

## Evidence

On the corpus, `vize lint` with `preset: "incremental"` before and after:

| Project      | `parser/jsx` before | After |
| ------------ | ------------------- | ----- |
| vuetify      | 136 (16 errors)     | 0     |
| naive-ui     | 31                  | 0     |
| vant         | 28                  | 0     |
| element-plus | 12                  | 0     |

`tests/_fixtures/differential/jsx/opaque-slots-*` freezes the compiled modules for
the four shapes. `tests/tooling/support/jsx-opaque-slots-runtime.ts` mounts each
module beside Babel's output under happy-dom and requires identical HTML that
includes the forwarded slot.
