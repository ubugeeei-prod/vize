# Native n8n branch-key storage review

Review base: PR #8156 head `66c2d37c417696880d47bff163738b133b8a97e6`.
The owning compiler/fixture lane records this decision on the Vize issue and
links it from the canonical decision record during consolidated integration.

The native branch-key comparison keeps earlier processed Simple keys in their
existing emitted branch walk. The ordered state has one `alloc::Vec` binding
and holds owned `vize_l0::String` values. Compound keys do not enter it; default
mode creates no entries. The `emit` category describes this document-sized
ordered emitter state. No parse, pipeline stage, public API, or serialization
is added by this inventory correction.

The prefix reparse helper was extracted to keep source files below 350 lines.
Its seven existing String uses must remain visible to the storage scanner.
Importing `vize_l0::String` directly in the helper exposes their actual storage
identity instead of relying on an untracked parent-module alias. The function
body and its allocation behavior remain unchanged.

| Production file                  | Reviewed String direct/bound | Reviewed Vec direct/bound |
| -------------------------------- | ---------------------------- | ------------------------- |
| `emit/prefix/rewrite.rs`         | 1/13 → 1/6                   | 0/0                       |
| `emit/prefix/rewrite/reparse.rs` | absent → 1/7                 | 0/0                       |
| `emit/vif.rs`                    | 1/2 → 0/0; remove stale row  | 0/0                       |
| `emit/vif/keys.rs`               | absent → 1/5                 | absent → 1/1 (`emit`)     |

Paths above are relative to `davinci/vize_l1_to_l2/src/`.
These are source inventory measurements, not runtime allocation counts.
The ordered key state was already present in the review base; these rows make
its actual source identity and the helper relocation reviewable.

| Whole inventory metric               |      Before |       After |
| ------------------------------------ | ----------: | ----------: |
| String files/direct paths/bound uses | 194/208/838 | 195/209/841 |
| Vec files/direct paths/bound uses    | 223/271/751 | 224/272/752 |

The L1-to-L2 scope changes String 112/124/502 → 113/125/505 and Vec
86/91/326 → 87/92/327. Arena Vec, SmallVec, dependency direction, instruction
ceilings, allocation gates, corpus exceptions, and skip allowances are unchanged.
Actual instruction/allocation qualification remains subject to the ordinary
source and merge-queue checks; source inventory growth grants no budget waiver.

Reviewed source SHA-256 identities:

- `emit/prefix/rewrite.rs`: `38938c816a762ad183ff0c86370a24732b64605e46c40105b6b0012d48b67c0b`
- `emit/prefix/rewrite/reparse.rs`: `cba3d55ffd4058171d5cf7fb186ac08b75050ff1bd8d2656bdd737336937ea66`
- `emit/vif.rs`: `bfc36cef244d8dfa1d102688cd01b0ddcc98f2e9801467dca90515301937789d`
- `emit/vif/keys.rs`: `2677b2085d1ee654160fcf3224f777511f23ab1151f09e1b87e728c884713b8b`

Validation: the existing storage policy and generated summary tests pass all
seven checks using the reviewed rows; Rust formatting and diff checks pass.
The full hosted compiler tests remain pending their separate compile corrections.
