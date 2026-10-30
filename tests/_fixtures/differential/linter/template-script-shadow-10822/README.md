# Script-setup shadowing (#10822)

The two complete original n8n files are copied byte for byte from revision
`e882e8a483f433facb47bab9b407d0ec00a81172`. Matrix run `41049126273`, artifact
`12974982174`, source `34b2a8f60f2a73df5736422761c6f5f08cd21fe4` reports six
false negatives and zero false positives for n8n, all `vue/no-template-shadow`.
The current `eslint-plugin-vue@10.9.2` oracle reproduces those same six exact
file/rule/message/range observations, using the matrix's pinned parser versions.

`cases.json` retains the full original source, all reference message fields, and
complete expected native packets. Native packet ranges come from the foreign
oracle's original identifiers; native messages/help remain Vize's contract.
No matrix packet or budget is rewritten. Both budgets remain zero.

The prior #8849 scope corpus deliberately recorded script exposure as unsupported.
Three new successor cases authenticate its exact unchanged sources and full old
zero packets before supplying the newly supported expected answers. Every other
old lexical, `v-pre`, sibling and type-only boundary remains the historical law.
The original frozen corpus is unchanged and its complete SHA-256 is checked.

Controls cover sibling and nested declarations, ordinary module/function locals,
nonmatching setup names, `v-pre`, and Unicode with CRLF. Only script-setup value
bindings and declared props are added; Options API exposure and erased type-only
names remain separate work. Full matrix qualification is still required.
