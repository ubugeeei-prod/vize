# Indentation append cost

The first [#6868](https://github.com/ubugeeei-prod/vize/issues/6868)
verification source `5e40274da99c9a25cd0b6c11649d57debf31ec45` measured
100 identical probes three times but failed 13 original instruction ceilings
in [run 36320906189](https://github.com/ubugeeei-prod/vize/actions/runs/36320906189).
Patina improved to 43809 / 538509. The iteration probe improved to 149089,
still above 148047. No failing ceiling is raised.

Against its physical L3 parent, the deep DOM compile added 273463
instructions. Raw exclusive attribution shows shared `Repr::push_str`
calls growing from 6 to 7897, while the previous `Buf::push` and
`Buf::newline` bodies disappear into callers. This is changed generated
call structure; the input and producer buffer source are unchanged.

The bounded production candidate appends indentation in constant chunks
of 32 spaces (16 levels), followed by the remainder. Division and modulus
bound the remainder to 0..15, so its byte length never exceeds 30 on either
pointer width. The newline remains first and uses the original operation;
spaces go through `Buf::push`. No helper, cursor, map, link or remark state
is changed. Appends use the existing output buffer, with no temporary buffer,
pipeline stage or serialization added. Exact allocation counts still require
the normal measurement gate.

Whole-buffer tests cover depths 0, 1, 15, 16, 17, 60 and 64 with Unicode
surrounding text, indentation transitions, deindentation at zero and helper
state. Local formatting, syntax parsing, assertion scanning and source growth
checks pass. Actual typed tests, complete compiler code/map and differential
fixtures, SSR/Vapor coverage and unchanged Linux 100-by-three measurement
remain pending. The iteration cost deficit is a separate unfinished fix.
Harness, benchmark inputs, stack protection and instruction ceilings stay
unchanged. The allocation result improved from 12 to 11, but its exact gate
stopped workspace tests; acceptance requires a separately reviewed downward
allocation ratchet after final-source evidence, never skipping that gate.
