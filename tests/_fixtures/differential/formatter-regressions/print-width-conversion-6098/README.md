# Formatter width-conversion observations

This finite corpus retains four existing historical inputs and their complete
100-column output oracles. `Original.vue.txt` is byte-identical to the original
deeply indented direct-call attribute from the expression-width #7876 corpus.
The catalog pins every input and existing output with SHA-256; old files remain
unchanged.

The initial source records the actual OXC default/range conversion, all three
API passes for each of twelve widths, and all configured/direct CLI processes
before checking any formatting results. It does not claim that the large-width
outputs are correct. The bounds include 65535, 65536, four 65536-plus-valid-width
residues and the largest `u32`, alongside ordinary widths.

The prospective fix must preserve the existing formatter fallback when a width
cannot be represented by the downstream type. It must prevent modulo narrowing
without adding a configuration cap, changing an old oracle or claiming native
adapter coverage. Actual hosted BEFORE and AFTER packets remain distinct.
