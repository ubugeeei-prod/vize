# CLI declaration slot-export successors

The current CLI declaration packets retain the exact original output except for
`export type Slots = $Slots;` becoming `export type { $Slots as Slots };`.
The emitter change in `1e49aa635bf256fec202a8d0ad1bd5e75f94cf8e` deliberately avoids
introducing a local `Slots` binding that can collide with authored declarations.

Both original CLI snapshot files remain byte-for-byte intact. New
`_slot_reexport` packets carry that single export-spelling change. Their complete
changed declaration strings were compared with the old/new full lines in the
actual Full Rust result for `8b86ad667fc51f979cfb736c9c763bde98895c09` (artifact
`13005399879`); all helper declarations, output paths, statuses and counts remain
unchanged. The tooling law pins the original SHA-256 values and requires each
successor's entire bytes to equal that one replacement.

The fallthrough CLI oracle is unchanged. Its existing display-only `$CheckTail`
normalization now also recognizes the captured Corsa rendering that truncates a
`$PublicComponentAttrs & {` tail before the closing `>`. The added path requires
the exact TS2353 diagnostic prefix, target-type prefix, truncated-type/source
boundary and complete diagnostic suffix. The whole authored diagnostic vector,
including its code, location and source, stays pinned. Other errors, arbitrary
types, missing boundaries and unexpected suffixes remain unchanged.
