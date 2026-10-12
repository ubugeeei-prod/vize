# Fixed MoonBit core archive

`core-0.10.7+bc794d341.tar.gz` is the unmodified original MoonBit core source
archive from `https://cli.moonbitlang.com/cores/core-0.10.7%2Bbc794d341.tar.gz`.
Its complete SHA-256 is
`06922d35dd94dc0baea7682295fe6084f0d854eed3b497cd6b527f77d899a7f5`
and its size is 1,449,345 bytes. This is the existing core hash in
`tools/nix/moonbit.nix`, retained from its authenticated fixed-output Nix store
archive after the original download URL returned HTTP 403.

The unchanged archive contains all 1,044 original regular files, including the
Apache-2.0 license at `core/LICENSE` and the original attribution notices.
It is redistributed as the exact source archive, without changing a member,
permission, tar header, or gzip byte. The checked source makes cold Actions
installation independent of the unavailable historical core URL.

The compiler is downloaded from the immutable
[`v0.10.7+bc794d341` archive](https://github.com/chawyehsu/moonbit-binaries/releases/tag/v0.10.7%2Bbc794d341).
The three Unix archive hashes equal the existing Nix pins. The Windows ZIP
adds an explicit whole-archive hash from that release and its checksum asset;
the archiver downloads the official ZIP, records the actual `moonc -v`, and
copies that same ZIP unchanged. This provenance does not replace the actual
Windows installation, exact compiler version, registry, and native smoke gates.
