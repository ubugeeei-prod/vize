# Newline prefix append

The private [#6868](https://github.com/ubugeeei-prod/vize/issues/6868)
candidate starts from `56fcfd91a048c7e02693f12fb49f2f920b99920a`.
Its [instruction run 36323473099](https://github.com/ubugeeei-prod/vize/actions/runs/36323473099)
measured 100 identical probes three times but failed 17 unchanged ceilings.
The event emitter measured 46508 against 46119. The accepted source's raw
profile records four `Buf::newline` calls; the current implementation appends
the newline and indentation separately. This motivates fewer producer appends,
without predicting savings or treating the failed source as accepted.

Append the newline plus the first 0..16 indentation levels from one constant
prefix through the existing `Buf::push`. Then append the remaining levels
with the existing 32-space chunks and remainder. The prefix byte length is
1..33. Subtracting the first levels cannot underflow; the remaining byte
length is still 0..30. Neither calculation multiplies the full depth, so it
stays bounded on both pointer widths for every valid `u32` depth.

The complete byte sequence remains one newline followed by two spaces for
each level. All appends use the existing destination buffer; helper, cursor,
source-map, link and remark state are untouched. No temporary buffer,
pipeline stage, serialization, allocator warmup or stack-guard change is
introduced. Depths 1..16 use one producer append rather than two. Depth zero
also uses `Buf::push`, so its generated cost must be measured too.

Keep both existing whole-buffer controls byte-exact. They already cover
depths 0, 1, 15, 16, 17, 60 and 64, Unicode surrounding text, indentation
transitions, deindentation at zero and helper state. Their actual default
workspace execution passed on the starting source; the changed producer's
typed tests, complete Code/maps/SSR/Vapor differential corpus, allocation
gates and all 100 probes three times remain pending. MacOS allocation
evidence is unknown. Inputs, windows, harness and instruction ceilings are
unchanged. Root review and the paired issue record precede composition or
runtime validation; this private candidate is not a performance claim.

Local Rustfmt 1.98 syntax/format, repository Markdown format, source growth
and strict assertion scans pass. A source/specification reconstruction agrees
with all output bytes for 8196 depths and verifies bounded arithmetic through
`u32::MAX` on either pointer width. This does not substitute for executing
the typed buffer controls or measuring the next exact source.
