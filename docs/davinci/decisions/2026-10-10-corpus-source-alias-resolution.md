# Read every logical corpus alias through its physical source

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), with the
[paired decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6093939583).

PR [#8361](https://github.com/ubugeeei-prod/vize/pull/8361), exact source
`5761bdf4a36f7cf5cb816d26eda69a38109499bf`, failed Check `38020153460`.
The ordinary SSR worker `114119212765` failed while reading
`BlurhashCanvas.vue` through forty `packaging/deb/root` aliases in the pinned
Jellyfin fixture. It retained Linux `ELOOP`, kind `FilesystemLoop`, raw errno
40 and the complete original path. The DOM aggregate then failed its required
observer precondition; that aggregate did not perform DOM comparisons.

The subsequent diagnostic read every one of the same 44,367 logical inputs
under both short and literal manifest-root spellings. Both completed all
177,468 Rust operations, every Node byte comparison and every pathname-prefix
observation. Its minimal fixture also read all 41 aliases. These results do
not explain the intermittent ordinary-reader failure. Preserve the earlier
[Linux IO diagnostic](./2026-10-08-canonical-corpus-linux-io-diagnostic.md)
and the failed original result; a passing replay does not identify its cause.

The strict source reader now resolves each path component before appending
the next, then performs one UTF-8 read of the resulting physical file.
Repeated ancestor aliases therefore do not accumulate in one kernel pathname
lookup. Canonical resolution of an individual link still rejects a genuine
self-referential cycle. Every resolution/read failure reports the original
logical path and OS error; there is no failed-read retry or alternate reader.

The collector is unchanged. Preserve every logical path, its order and alias
multiplicity: 44,367 total inputs, including all 5,576 Jellyfin aliases and
148 pinned gitlinks. Existing native collector, physical/Git vectors, source
digests, SSR/Pug/DOM comparisons, exact observer custody and merge-queue gates
remain mandatory. No input, oracle, accepted-error policy, production compiler
behavior, pipeline stage or resource ceiling changes.

An independently constructed eighty-alias path exceeds the ordinary kernel
lookup limit but must return the complete BOM/Unicode/CRLF source through the
new reader. The same fixture requires every unchanged collected alias to
return those exact bytes. A genuine self-link must retain its complete failure
diagnostic. Existing missing-directory/entry/source, invalid UTF-8, sorted
vector and byte-retention controls remain intact.

TODO: qualify the exact source on Actions, merge through the protected queue,
and refresh #8361 against actual main before its own full qualification and
delivery. This bounded reader repair does not close the CI roadmap, explain
the historical intermittent errno, or establish #7951 runtime/release closure.
