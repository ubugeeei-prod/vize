The adjacent gzip preserves complete, unmodified Git blobs for all 37 generated
version changes and the unchanged native workflow in the official 0.436.0 cut:

- C: `6d26d84b4240e9356cf5078b6d277e181d311a42`
- H: `3390cb6044d53d655a9d64e112d2618375cff5ca`
- Original JSON SHA256: `2b98c5e519c8584a3bc625210c03ce6ea7611ab973f871012eb2510222e52976`
- Gzip SHA256: `b52db3ad5407614c9fd985a28f22083808d9d17852583ce6fd57153b34164cf0`

Each original before/after blob has its own SHA256. The custody control
decompresses without rewriting any byte and reconstructs disposable Git commits
with explicitly different identities. This verifies exact metadata and complete
recipe selection; it grants no hosted compiler or release acceptance.

`native-current-recipe-8231.sh` preserves the complete current step from signed
actual main `b41811e3b33676f68a0843a693c9ed4898f2e86a`, including the nine declared
CLI targets and every downstream command/flag. Its SHA256 is
`397a7119bdeab7cecdb7e1f7c5ed62b8df1232005fcd3d538ac6f20bf761fb13`.
The original full bytes and digest remain immutable controls.

`native-current-recipe-8253.sh` independently authors the complete ten-target
successor by adding only `--test check_tsconfig_bom_cli` after the last existing
target. All original nine targets and downstream bytes remain unchanged; the
fixture law verifies this complete additive relation. Its SHA256 is
`b2bd7c08e90ca1625044d09cbe221c22f8a8344be8c4347854acb73a66dfc05f`.
The current wrapper must execute the complete successor once, and a version-only
source that deletes its new target is rejected. Neither fixture is derived from
an executing producer's response or grants hosted native acceptance.
