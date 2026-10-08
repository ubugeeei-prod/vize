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
