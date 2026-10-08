# Content Mapper quoted event definition regression

Tracker: [#4075](https://github.com/ubugeeei-prod/vize/issues/4075).

`input.vue.txt` is a byte-exact copy of the original
`content_mapper_project/src/CallSignatureChild.vue`, which failed in scheduled
run 37769763871 at main 8816b0d5. Its literal definition must retain the complete
source bytes `[49,57)` (`2:10–18` in zero-based UTF-16 coordinates).

`cases.json` fixes that original complete protocol endpoint and 16 independently
authored whole protocol results before hosted execution. They cover verified
double/single quote pairs and unchanged fall-through for absent, reversed,
mismatched, escaped, empty, boundary, sub-span and feature-mismatched entries.
Generated offsets in these finite controls are absolute byte offsets.

The direct original regression runs the real Content Mapper producer. The
existing stock pinned-tsgo event-forms test remains unchanged and qualifies the
actual definition/rename workflow. Shared native #4075 inputs and whole-packet
assertions are unchanged. These fixtures grant no public or broad issue closure.
