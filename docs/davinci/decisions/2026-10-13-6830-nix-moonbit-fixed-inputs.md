# Keep Nix MoonBit inputs available with the original byte authority

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The full release Check at immutable `2817a0aa13518dd46c16bc8bf56ac831f68c24dc`
failed its original Nix flake check before toolchain installation:
[job 115082649416](https://github.com/ubugeeei-prod/vize/actions/runs/38331701024/job/115082649416).
The pinned MoonBit core URL returned HTTP 403 through all original fetchurl
retries. Bounded subsequent GETs of both encoded and literal-plus core URLs,
and the Linux x64 compiler URL, returned Amazon S3 `AccessDenied` through
CloudFront. These responses establish the observed transport refusal, not its
provider-side cause or universal platform availability.

Nix still downloaded these inputs from the upstream URLs directly. The existing
setup-moonbit action already authenticates a tracked original core archive and
three same-byte Unix compiler archives on a fixed historical mirror.

Keep Nix's existing fetchurl derivations and every fixed hash. Route core through
the tracked asset whose full 1,449,345 bytes have SHA256
`06922d35dd94dc0baea7682295fe6084f0d854eed3b497cd6b527f77d899a7f5`, exactly
the original Nix SRI `sha256-BpItNd2U3Auup2gilf5ghPDYVO7TtJfNa1J/d9iZp/U=`.
For each existing Unix compiler platform, retain its original URL first and add
only its already-reviewed setup-moonbit mirror second. Preserve version-file
authority, derivation names, complete compiler hashes, platform selection,
ELF adjustment, all three core bundle commands and the real `nix flake check`.
Changed or missing payloads must remain failures; no new compiler version,
deadline, qualification skip or replacement runtime is introduced.

The source regression resolves the actual core route, verifies its complete body
and unchanged configured hash, refuses a real changed byte, compares every
compiler URL pair with the existing authenticated transport inventory, and
requires the original flake-check command. The previous installer controls stay
intact. Ordinary source checks, an exact-source hosted full Check including the
actual Nix job, protected queue checks and actual merge remain required.

The independent editor-host refusal is preserved separately. Its pinned VS Code
1.107.1 downloader exhausted upstream response-aborted retries before either
actual host suite ran, then reached its existing 300-second process deadline.
A successful bounded range GET is not a whole archive or host qualification.
This Nix change grants no editor acceptance, release promotion, performance gain
or same-H retry authority. The release operator owns those decisions.

TODO: record exact source/full-Nix/protected/actual-merge results after Actions.
