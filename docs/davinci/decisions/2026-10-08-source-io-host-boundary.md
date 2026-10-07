# Source IO host boundary

Owning issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834), with the
[paired decision](https://github.com/ubugeeei-prod/vize/issues/6834#issuecomment-6049188718).
This is a bounded continuation of its Stage 1 storage ownership work after
#6833, using genuine signed main `381637687961ac92608ce53b30464d2be8b3e6b0`.

`vize_l0::source_io::decode_utf8` owns borrowed byte validation. Its existing
short-input and SIMD paths preserve authored bytes, pointer identity and the
standard `Utf8Error` fields and spelling. The filesystem adapter belongs to
`vize_carton::source_io`, which re-exports that same decoder for legacy users.
Its existing `read_to_string` reads an owned host buffer, validates it once,
and returns the same buffer without an additional copy. Keep standard OS
errors and invalid-UTF-8 behavior unchanged.

The first commit moves the two complete source IO files from L0 to Carton
without changing their bytes. A separate integration commit restores the
existing decoder and its three laws to L0, leaves the filesystem function
and its law in Carton, and updates actual CLI, NAPI and Maestro host callers.
The scripted move and integration fail on changed anchors. Carton's ordinary
module shadows its previous glob facade; level crates gain no dependency on
Carton and keep the same borrowed decoder. This adds no provider, pipeline
stage, parse, serialization or product route.

Existing benchmark source and inputs, all numeric ceilings, corpus bytes and
runtime laws remain unchanged. Consumer inventory updates follow the actual
import moves. The foundation storage inventory removes only the three
filesystem-owned imports from L0; its strict scanner and remaining rows stay
in force. The existing text-boundary Actions lane selects both source IO
owners and runs both crates' laws, while retaining the actual-file benchmark
and its Clippy check.

Hosted qualification must execute the exhaustive one/two-byte and SIMD-boundary
law, Unicode/truncation/overlong/surrogate cases, deterministic random buffers,
pointer identity and the complete real-file/OS-error law. Fresh source/native
checks, unchanged differential corpora, original protected instruction ceilings
and actual signed merge remain required. Local extraction and static checks
grant no runtime or measured performance credit.

Whole #6834 remains open. This slice does not establish a `no_std` build or
complete platform isolation: timing, profile stacks, allocator/pool globals,
path/config and other host dependencies retain their own remaining work.
Product fix-history gates and native/default graduation requirements are
unchanged. Release and installed-consumer validation remain separate.
