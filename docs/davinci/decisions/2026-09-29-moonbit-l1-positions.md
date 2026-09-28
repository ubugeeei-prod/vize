# MoonBit source positions in L1 (2026-09-29)

Tracked in the [#6841 ownership checkpoint](https://github.com/ubugeeei-prod/vize/issues/6841#issuecomment-5872818335).
`moonc` reports one-based lines and columns in Unicode scalar values. L1
owns the text-to-byte correspondence for both authored source and generated
MoonBit text, so the existing position index moves without behavior changes
to `vize_l1::lang::moonbit::lines`.

The first commit is a 100% file rename. A separate adapter commit imports
`alloc::vec` for L1's `no_std` build, exports the new module, and keeps
`vize_dialect_moonbit::lines` as a thin re-export for current callers. The
original round-trip tests run at the L1 location. This adds no L1 dependency
on Croquis, the old dialect crate, or the extension packages.

The index retains one owned `Vec` of line-start byte offsets, sized by the
source's newline count. The reviewed L1 analysis row in
`docs/davinci/plan/storage-inventory.tsv` records one direct import and one
bound use; no fixed inline bound applies to authored text.

The L1 `moonbit` feature exports this language module only for MoonBit
consumers; `vize_dialect_moonbit` enables it on its L1 dependency. Ordinary
Vue builds do not compile the unused position index. The protected queue
measured unchanged behavior but tiny shared lowering instruction regressions
across DOM, SSR and Vapor when the module was unconditional. Callgrind assigned
the deltas to common L1→L2 lowering functions, not to `Lines`. The feature
keeps the default graph focused while feature-enabled L1 tests and MoonBit
checks cover the new API. The 100-probe instruction gate must still pass
without increasing any ceiling.

The other old MoonBit files remain in their current crate for now. Its SFC
split still calls legacy Croquis; `vize_l1::container::Vue` is a `todo!`
skeleton under #6837. The guest L1 surface page and L4 projection use the
old Dump implementation while `vize_l0::dump` remains a skeleton under
#6833. These prerequisites must be implemented before their adapters can
move without a legacy edge or a broken product path.
