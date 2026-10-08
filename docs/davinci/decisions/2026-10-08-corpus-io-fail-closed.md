# Fail closed on corpus enumeration and SSR source IO errors

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), with the
[paired decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6051619211).

The historical source-runtime SSR sweep reported 44,366 files while its
same-tree integration sweep and retained native path vector reported 44,367.
Another historical formatter run reported the same SSR gap while its DOM
sweep processed all 44,367 files with zero unreadable inputs. A later formatter
run processed all 44,367 SSR files without changing these reader authorities.
The retained complete path/blob manifests and gitlink/status inventories
agree; they do not retain the omitted SSR path, its then-on-disk bytes, or an
IO error. The precise cause remains unproved.

SSR increments its file counter before parsing or compiling each input.
Consequently, the observed omission occurred before that comparison boundary.
The shared collector silently returned on `read_dir` errors and discarded
failed directory entries, and the SSR loop silently continued when a selected
file could not be read as UTF-8. Pug's reported file count is its collected
vector length, and its source reader also silently skips failures; its full
count does not establish that every collected source was read successfully.

The shared collector now fails the calling test when directory enumeration or
an entry fails. Diagnostics retain the operation, containing directory,
original error display, `io::ErrorKind`, and raw OS error. An entry error does
not expose the failed entry's filename through the standard library, so that
diagnostic truthfully identifies the containing directory. SSR uses a shared
strict source reader and fails on a selected file's missing, unreadable, or
invalid UTF-8 contents, reporting the complete selected path and original
error. It never presents that omission as a completed sweep.

The collector's sorting, append order, directory and symlink classification,
`node_modules` and `_git-worktrees` exclusions, environment-unset behavior,
canonical inventory reconciliation, and smoke scope remain unchanged. Both
SSR comparisons receive the same complete source bytes. All existing battery
inputs, corpus files, golds, report counters, finite 44,367-file requirements,
instruction ceilings, and CI steps remain unchanged. This adds no production
compiler behavior, pipeline stage, or serialization.

Six test-support controls retain a complete sorted vector with both excluded
trees and ordinary non-SFC inputs, a missing directory, an injected failed
directory entry, a selected file removed after collection, invalid UTF-8, and
byte-exact readable BOM/astral/CRLF source. Failure controls inspect the complete
panic payload, including the original OS error when one exists.

The [paired harness correction](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6051767643)
records the first source run, which compiled these six controls successfully but failed before
the canonical comparisons: the existing native evidence harness extracted only
the collector body and omitted its two new private helper definitions. That
existing harness now carries the exact source helper definitions and refuses
missing or duplicated helpers. Capture and validation reconstruct the same
harness; historical sources without helpers retain their original harness.
The complete original 778-byte collector from signed actual `b41811e3` remains
a frozen test control with its entire kernel, alias, UTF-8, order, and byte
assertions. An additional real execution compares the current complete raw
NUL path vector against that original and the independent physical/Git vectors,
and requires a missing-root diagnostic with nonzero exit and no success vector.
No original control, fixture source, vector, or finite corpus requirement is
weakened or replaced by this harness integration.

TODO: metadata-error and symlink policy, and uniform fail-closed reads in other
corpus lanes, remain separate work. This bounded installment preserves those
policies and does not close the CI roadmap or identify the historical errno.
Fresh exact-source Actions and the protected corpus gates must qualify the
change; prior successful runs are historical evidence only.
