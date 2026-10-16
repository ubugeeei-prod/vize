# Numeric `v-on` modifier compatibility

Vize's `vue/no-deprecated-v-on-number-modifiers` follows the diagnostic selection and key
replacement behavior of **eslint-plugin-vue 10.9.2**. This resolves the isolated numeric-modifier
mismatch in [#9204](https://github.com/ubugeeei-prod/vize/issues/9204). It does not establish
full-configuration or published-package acceptance for n8n [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

For each `v-on` directive in Vue 3, the rule selects the **first** modifier for which JavaScript's
`Number.parseInt(name, 10)` produces a finite integer. It then reports that modifier only when the
parsed number is outside 0–9. The selection happens before the 0–9 exemption:

| Modifiers                                     | Reported modifier      | Replacement |
| --------------------------------------------- | ---------------------- | ----------- |
| `.13.27`                                      | `13`                   | `enter`     |
| `.27.13`                                      | `27`                   | `escape`    |
| `.stop.13.27`                                 | `13`                   | `enter`     |
| `.0.13`, `.9suffix.13`, `.0x13.27`, `.1e2.13` | none                   | none        |
| `.13suffix`, `.+13`, `.0013`                  | the entire modifier    | `enter`     |
| `.999.13`, `.-1`                              | first numeric modifier | none        |
| `.f2.27`                                      | `27`                   | `escape`    |

Numeric prefixes are intentional provider compatibility. Values 0–9 are ordinary digit-key aliases
under this contract, even when a later modifier contains a deprecated key code. Values with no known
named-key mapping still produce a diagnostic, without a fix. Infinite numeric overflow is skipped
before selecting the next modifier. The rule preserves Vize's existing Vue 2 and petite-vue exemption.

Known keys carry a native `Fix` containing one replacement edit over the complete physical modifier.
This includes Enter, Escape, navigation keys, modifier keys, letters A–Z, F1–F12, Num Lock and Scroll
Lock, using the provider's spellings. The leading dot, event name and other modifiers are preserved.
Multiple numeric modifiers can therefore require multiple ordinary fix passes, as in the provider.
Native diagnostic and fix ranges use UTF-8 byte offsets in the containing SFC. The tests additionally
check these ranges with a preceding Japanese script string and a multiline template.

Vize retains its own translated diagnostic/help wording, native severity representation and fix
explanation. The matched fields are rule identity, number and order of findings, configured error
severity, physical location, replacement edit range/text and absence of a fix for unknown keys.
ESLint's messageId and UTF-16 position representation remain provider-specific.

## Reproduce the independent provider matrix

The adjacent [probe](numeric-key-modifiers/probe.cjs) runs only this numeric rule against 96
authored inputs (95 rule cases and one malformed parser control); [provider.json](numeric-key-modifiers/provider.json) retains every complete
`ESLint.lintText` result without filtering messages or fixes. It records ESLint **10.4.1**,
eslint-plugin-vue **10.9.2**, vue-eslint-parser **10.4.1** and the installed
@typescript-eslint/parser **8.65.0**; this isolated template probe does not enable the TypeScript
parser. It covers all reported mapped key codes, digit aliases, numeric prefixes/signs, first-match
ordering, unknown values and finite/infinite large integers.

With those exact dependencies installed in a separate provider directory, regenerate the packet:

```sh
node docs/acceptance/numeric-key-modifiers/probe.cjs /absolute/path/to/provider-directory
```

The rule's `qualifies_complete_provider_matrix_and_malformed_control` Rust test consumes every
recorded input. It compares diagnostic locations and replacement edits for all 95 rule cases.
The retained malformed `@keyup.="submit"` control explicitly asserts a whole-output boundary:
ESLint reports nothing, while Vize reports one `parser/template` error with no fix. This input is
not qualified as matching; no messages are removed from either result to disguise the difference.
Existing named-key, shorthand/longform, combined modifier and petite-vue tests remain in place.
GitHub Actions runs the current-source Rust tests; recording provider output alone is not proof
that a native build passes, and this matrix does not change the status of the broader n8n controls.
