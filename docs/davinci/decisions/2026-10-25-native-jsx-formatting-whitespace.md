# Native JSX formatting whitespace

Issue: #10488. Parent native owner: #10468.

The canonical static markup producer now admits multiline text consisting only
of ASCII spaces, tabs and CR/LF when the original parser value equals its actual
authored token. This formatting-only family creates no JSX text child. Single-line
spaces remain real canonical Text ops; single-line tabs, meaningful multiline
text and entities retain their typed whole-view normalization refusal.

The existing original Program records and their complete source ranges stay
owned. Preparation and emission share one allocation-free predicate. Removed
formatting tokens mint no canonical node, no synthesized buffer/map, and no new
File storage. Ordinary source parsing, File facts and target admission are unchanged.

The frozen 15-case oracle runs the pinned actual JSX transform and mounts every
result against Vue 3.5.42. It compares complete HTML and DOM trees, retains complete
transformed modules and source hashes, and obtains independent authored byte spans
from the original reference parser. Ten native-admitted cases and five explicit
refusals cover LF, CRLF, bare CR, tabs, blank lines, nested elements, adjacent
comments, meaningful inline spaces and entity controls. Rust tests compare every
native canonical node, attribute/value, namespace, original span and whole refusal
packet for both JSX and explicit-module TSX. The new native law fails on the parent
before this source change. The reference runtime mounts in the pinned DOM harness;
this is semantic evidence rather than a browser rendering or LSP latency claim.

The phase probe accepts a third `layout` argument. Its committed receipt alternates
parent compact input, child compact input and newly admitted multiline input;
instrumented source buckets run separately. Preserve the initial native-owner
receipt as historical evidence. New native admission does not establish a compiler
throughput or universal speed result.
