# MoonBit L2 expression capability move (2026-09-28)

The maintainer authorized #6832 and L1–L4 structural work in parallel in
[#6826](https://github.com/ubugeeei-prod/vize/issues/6826#issuecomment-5871526916).

The first #6841 move places the MoonBit `ExprDialect` implementation and
lexical scan under `vize_l2::lang::moonbit`. The old
`vize_dialect_moonbit::dialect` path is a temporary re-export so the typed
guest and current tests keep using the same semantics during migration.
The source-file move is its own rename-only commit.

This is only an L2 responsibility move. The `vize_dialect_moonbit` package,
its Croquis-backed SFC split, typed projection, and checker host remain.
The SFC split needs the native L1 container boundary (#6837), and the
projection and checker host need their level capabilities. #6841 stays open
until the package is dissolved and the native path is verified.
