# Original native component owners

Issues: #6835, #6836, #6838 and #6844.

L1 now has a move-only `markup::NativeComponent` with private original allocator,
checked SourceBlock and complete once-parsed ComponentParse fields. Construction
calls the existing stock native markup parser once. Readonly access preserves the
original recovered tree, diagnostics and unsupported heads. Consuming transfer
of the raw carrier discards this source authority; a caller cannot reinsert a
mutated or independently parsed carrier.

The existing root and element child slices produce opaque, non-Clone projections
of the actual direct parent, original child and ordinal. Element projections
expose their original child and Attribute iterators; each Attribute projection
retains the complete original Attribute, same element and real header ordinal.
Constructors are private. Borrow lifetimes remain tied to the original component
owner, including empty roots, and prevent moving or dropping that owner while a
projection remains in use. No tree clone, extra traversal, marker allocation or
new pipeline stage is added.

The unchanged storage scanner records only the new test-owned lists (one alloc
Vec import and two bound uses) in the existing per-file inventory. Existing
production storage rows and the complete Croquis census remain unchanged.

The private actual-source check runs all 321 L1 unit laws, production and whole
unit strict Clippy with the committed test configuration, six real privacy and
lifetime compilations, and a positive immutable-owner/mutable-recorder field
split. The original no-std fixture and internal enum-copy lint failures remain
preserved separately. These are local whole-L1 checks with exact parser and
dependency inputs, not Actions or whole-Cargo acceptance.
The original input manifest has 148 rows for 146 unique files, including all
141 committed L1 Rust files. Two baseline native files appear twice with
identical hashes; their duplicate labels do not add authored sources or tests.

A Component parsed from an arbitrary checked block does not prove Descriptor
template selection, script/no-script profile, complete File construction or
native body admission. The lower combined owner and guarded same-walk completion
remain required by the [ownership decision](https://github.com/ubugeeei-prod/vize/issues/6838#issuecomment-5963043881).
The sole SFC pending row must retain that combined owner without duplicating the
component. Public provider delivery, lower guards, native control output,
product history, defaults and instruction/allocation gates remain unfinished.
