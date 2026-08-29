# The systems this repository supports, shared by the flake's `systems` and by
# `meta.platforms` on the packages, so the two cannot disagree about what is
# buildable.
#
# x86_64-darwin stays on the list: the pinned nixpkgs still ships it, and this
# is a restructure, not a narrowing of what the flake builds.
[
  "aarch64-darwin"
  "aarch64-linux"
  "x86_64-darwin"
  "x86_64-linux"
]
