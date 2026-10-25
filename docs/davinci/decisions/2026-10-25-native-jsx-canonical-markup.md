# Original-owning JSX canonical markup

Issue: #10468. Related product contracts: #6848, #6871.

`JsxFileProducer::finish_markup` is an opt-in producer for a bounded static HTML
family. It consumes the actual original Program observation, the same completed
File facts and the already recorded JSX preorder. It builds the canonical Region
in that File before sealing; it introduces no parser, second AST walk, Relief
projection or caller-supplied source/fact pair. Existing `finish` and JSX output
admission stay unchanged.

The opaque `JsxMarkupFile` retains the original owner and exposes its immutable
canonical File. Native JSX and explicit-module TSX support static intrinsic HTML
elements, authored static attributes and single-line text. Original empty JSX
containers create no rendered node; their comments remain in the retained
Program. Ordinary script expressions retain their complete original File facts.

Whole-view preflight refuses components, foreign namespaces, fragments, spreads,
dynamic children/attributes, special Vue properties and values requiring entity
or multiline JSX normalization. A later unsupported root leaves earlier roots
unpublished. File/factory failures retain their actual original rejected owner;
no rejection acquires a native markup success marker. This is shared analysis
authority, not complete JSX module or target output admission.

Exact tests pin the complete canonical tree, attribute values/ranges, original
Program storage and all retained File rows. LF/CRLF, astral text, a nonzero embedded
source block, owner movement and foreign-owner isolation have independent range
witnesses. The common complexity pass consumes the genuine native Region.

`jsx_markup_profile` reports parse, original File walk, canonical publication and
shared analysis separately. Optional profiling exports preparation, emission and
File sealing buckets. Build before measuring; exclude competing compilation and
keep instrumented runs separate from uninstrumented timing. The accompanying
receipt records the local optimized probe only, with no end-to-end compiler,
editor latency, legacy comparison or universal speed claim.
