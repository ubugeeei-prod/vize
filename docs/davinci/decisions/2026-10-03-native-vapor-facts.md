# Native static Vapor decisions in the shared walk

Tracking: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839), [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The Vapor collector joins the existing canonical enter/binding/leave traversal
only for `TargetPolicy::Vapor`. It retains original L2 Element/Text/Comment
references and complete root part ranges. Every node still receives the original
neutral decision; DOM and SSR policies keep their existing facts and hooks.
There is no extra tree traversal, parse, intermediate string, legacy IR or
serialization between levels. The unchanged shared error enum moves into its
own module in a separate factoring commit, keeping the shared builder under the
350-line source limit and preserving its public API.

The first family admits finite HTML tags whose trees do not trigger browser
table/select/paragraph recovery in the admitted well-nested trees: div, span, section, article, main,
aside, header, footer, nav, small, br, hr and img. Generic id/title/role/dir/lang,
data-* and aria-* static attributes retain their actual decoded values. The
collector records actual void closing eligibility, original root order and the
single non-comment element eligible for root-template attribute fallthrough.
All dynamic operations and bindings, other tags or namespaces, special/property
attributes, duplicate attributes, void children and unsafe comments refuse with
original node/span diagnostics. This finite family does not claim complete HTML
semantics or native effects/control-flow.

Decoded text must be stable under both the current native preserve-whitespace
contract and Vue's default condense contract: empty, leading/trailing space,
repeated ASCII spaces and tab/form-feed/CR/LF/NUL refuse. Unicode non-ASCII spaces
remain original text. Attribute CR/NUL and unsafe comment bytes also refuse;
unknown browser normalization is never silently admitted by changing the oracle
options. L4 must encode text/attributes once and prove the real Vue 3.6.0-rc.9
runtime, independently of the DOM/SSR lanes' Vue 3.5.35 authority.

Root text beginning with `<` refuses in the same shared walk: the actual static
runtime template helper treats that spelling as markup. Ordinary root text uses
the helper's raw text branch so hydration adopts the server Text correctly;
direct `createTextNode` mounted correctly but detached text during hydration and
failed on unmount in the pinned runtime. This target-specific boundary does not
change neutral text or any other target's facts.

`NativeVaporFileAnalysis` privately retains one completed immutable File and
derives its only artifact internally. Incomplete/interrupted files fail before
the walk. Root lookup checks actual ownership in constant time; equal node IDs
from another analysis cannot supply output parts. Compile-fail laws retain the
File through the analysis. Genuine once-lowered SFC/File laws validate pointer,
decoded attribute, root-order, target-isolation and original refusal semantics;
handcrafted File values are not the acceptance denominator.

A public neutral File builder can deliberately attach different text to equal
source spans. The generic File analysis therefore remains diagnostic-only and
cannot authorize native target output. `NativeTemplateVaporAnalysis` instead
retains the moved, non-cloneable original `NativeTemplateView` exactly like the
existing DOM entry. Its private fields derive the File internally, preserve the
actual original lower owner and invoke the same sole decision producer. Neither
a neutral analysis nor a complete manually populated File can promote itself;
three compile-fail laws and a real selected original-template law pin this
boundary. The target consumes only this original completion capability.

This provider is not an emitted product. The actual L4 target, complete module
and original-source maps, pinned runtime/reference execution, original SFC
target entry, native effect/control families, compiler fix-history #6880 and
default product migration remain unfinished. Hosted exact-head Actions and
protected unchanged instruction/full queue acceptance must reach actual merge.
