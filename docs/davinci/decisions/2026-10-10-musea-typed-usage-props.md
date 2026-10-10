# Musea copied usage prop values

Tracking issue: [#8377](https://github.com/ubugeeei-prod/vize/issues/8377).

The props editor's Usage code must pass the edited values to a Vue component
without changing their types. Keep ordinary string, number and boolean output;
emit an explicit empty string so it overrides a component default. Escape text
and binding expressions for the surrounding HTML attribute, including entity
spellings and authored CR/LF/tab characters. Use bound JavaScript expressions
for arrays, objects and null. Preserve negative zero and object keys named
`__proto__` rather than letting an object literal change its prototype.

The differential fixture retains text, nested arrays/objects, null, empty text,
entity spellings, quotes and a nested `__proto__` key. Source regressions compile
and render the generated Vue template and compare the complete received props.
The existing Chromium gallery contract edits real controls, copies Usage code,
compiles and executes the exact clipboard template, and retains the full copied
bytes, observed values and a screenshot in its existing artifact.

The existing Musea browser workflow runs this regression; full source and
protected merge-queue Actions must pass before delivery is complete. Successful
publication remains a separate release obligation.

Separate props-editor follow-ups discovered during this audit remain unfinished:

- `usePalette.load` does not prevent a late response for the previous art from
  overwriting the current art's state during rapid navigation.
- Removing a palette prop and adding the same name as a custom prop leaves that
  name in `deletedPaletteProps`, so it never reaches the preview; saved-state
  restoration also rejects the replacement as a palette-name collision.
- Add Prop accepts arbitrary names without validation. Names containing spaces,
  quotes or Vue directive syntax can still produce invalid or unintended Usage
  code. This change preserves values for ordinary Vue prop names; arbitrary
  custom-name support requires a separate keyed `v-bind` contract.
