# Reuse fresh lexical import scans in default checks

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
Decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5975590068).

Default `vize check` preparation walks the complete source graph before and
after adding hidden ambient declarations. The existing session already shares
package lookup and registration facts, but each walk reads and scans every
source again. Referenced default programs repeat this preparation independently.

Keep a source occurrence cache in those repeated-walk sessions. Each access
still reads the complete current file. Only identical source bytes reuse the
previous lexical scan; an edit replaces it, and a failed read evicts it. Cached
occurrences retain their individual contextual, import or require mode and the
existing lexical treatment of comments, raw SFCs and incomplete syntax. Import
targets are resolved again, so creating a previously missing relative target
does not require changing its importer. The cache adds no parser or pipeline
stage and does not change diagnostic scope or ambient registration rules.

Source retention is enabled only for known repeated default-program walks.
One-pass explicit scopes keep the existing owned occurrence vector and do not
retain source bytes or copy occurrences into a cache. Retained sources and
occurrences live only as long as the preparation session. This trades bounded
session memory for repeated scanning; it does not reduce disk reads.

The regression suite covers same-session same-length edits with restored mtime,
missing-target creation, failed reads and recreation, occurrence-mode and
lexical parity, and 500 SFC roots sharing one module. The latter two-walk case
must execute 501 scans and reuse 501 scans, rather than scanning 1,002 times.
`--profile` records `check.import.scan.calls` and `check.import.scan.reused`.

The fail-closed Check Benchmark Gate's pinned invocation selects an explicit
dot path. It checks regression safety for that path and does not measure the
default repeated-walk benefit. Acceptance needs exact-head Actions and separate
default-invocation measurements. This preparation slice does not establish the
10x full-command goal; that criterion in #7698 remains open.
