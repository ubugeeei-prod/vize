# Glyph joins the dependency-direction gate

Issue: [#6851](https://github.com/ubugeeei-prod/vize/issues/6851).

The existing dependency-direction gate protects physical level crates and their
normal transitive dependency paths. It did not start from product crates, so a
product could add or regain a legacy Armature, Relief, Croquis, or Atelier
dependency without failing that gate.

Admit `vize_glyph` as the first protected product root. Its current normal
dependency graph has one measured legacy boundary:
`vize_glyph -> vize_atelier_sfc`. Pin that exact Cargo declaration in the
product allowlist at `d9bab8d2708f66e53606b5ad29e42f2a471de51d` and allow the
list only to shrink. Removing the dependency without removing its permission
fails as stale; adding a direct or transitive legacy path, changing the alias,
target, optional flag, or workspace identity fails as unlisted.

This slice does not claim that Glyph has completed its L1 migration, and it does
not protect Patina, Maestro, or Canon yet. Those products remain follow-up work
under #6851. The ordinary source-head Actions and protected merge queue remain
the acceptance boundary.
