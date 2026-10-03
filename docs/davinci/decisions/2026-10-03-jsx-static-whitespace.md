# Native JSX static whitespace

Tracked in #6840 and #6829 after the original-expression child #7561 of
native Stack #7504. The provider literally merged at `134c4ad549ce` on
2026-10-03 at 12:59:16 UTC. This bounded target source is separate and has no
native execution or merge credit yet.

## Decision

The actual retained L2/L3 Text and AttributeString values remain unchanged.
The L4 target normalizes CRLF/CR/LF and TAB from those owner-bound values,
using the pinned Babel 7.29.0 / Vue JSX plugin 2.0.1 / Vue 3.5.35 policy.
No source reparse, AST visitor, metadata table or pipeline stage is added.

The transform replaces TAB with ASCII space, trims leading ASCII space on
non-first lines and trailing ASCII space on non-last lines, then joins
surviving lines with one space except the last original nonempty line.
U+2028/U+2029 and nonbreaking spaces remain actual text. Entity decoding is
a separate unimplemented boundary and keeps the existing typed refusal.

Normalized-empty Text rows remain in the neutral owner but are omitted from
target children and helper selection using the actual normalization result,
never a proxy classification of raw whitespace. A childless intrinsic uses null,
preserving the complete pinned VNode flags. Single-line spaces and TAB still
produce a text child. Quoted attributes may retain the normalized empty
string. Existing File helper collision facts and checked runtime vocabulary
remain authoritative; original comments keep their source custody.

Unchanged values avoid an additional normalization allocation. Changed values
use a temporary compact string at the existing writer boundary, with the
same original span for links. Source-map resolution stays in the shared
already merged standard-line-terminator provider; it is not duplicated.

## Prepared evidence and remaining work

Eight complete original sources retain pinned upstream transforms, maps and
nine actual upstream render executions. They distinguish normalized-empty
text, nested CRLF text, CR attribute values, a single TAB, Unicode separator
and nonbreaking text, empty attributes with comments, the exact earlier LF
refusal input, and genuine helper collision/read facts. The earlier LF input
moves from a refusal expectation into this complete admission fixture;
entity and component-child refusals remain explicit. All eleven original
module fixtures and three original-expression fixtures stay byte-exact.

Strict whole native code/map fields are pending. The hosted Rust law must
retain complete captures from the actual sole parse, walk, completed File,
moved owner and same L3 rows; manual raw-value/helper facts and complete
native code/maps stay exact assertions. The independent Node judge checks
actual mounted trees, full upstream transforms/maps, every native map
anchor's UTF-16 bounds and complete ordered comments.

No local Cargo/rustc/npm install, binary or Rust probe is used. Publish only
from the actual #7561 merge and fresh-main integration, obtain exact-head
hosted native observations without weakening assertions, then require fresh
source checks, protected full/all-100 candidate acceptance and actual merge.
Entities, component slots/children, TS erasure, compound JSX, dynamic
attributes, wider grammar and product/default replacement remain unfinished.
