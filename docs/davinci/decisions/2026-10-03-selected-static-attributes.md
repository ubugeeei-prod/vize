# Selected original ordinary static HTML headers

Issue: #6838. This is a bounded successor to the genuine selected original
HTML body provider in #7511.

## Decision

The existing private original Element route now completes ordinary static
attributes before invoking the checked File Element factory and its private
original body. The public root walk still accepts only an authentic ordered
`NativeChild`; callers supply no header, attribute vector, completion receipt,
factory, tag, namespace or body.

The private `body/header.rs` helper consumes `NativeElement::attributes()`
once. Each full original `NativeAttribute` is checked against the same
Component, Element pointer, next ordinal and actual original surface token.
Only the real iterator's normal end returns a completed canonical vector.
No Element or body begins when any header member refuses, even after valid
preceding attributes.

Bare attributes retain `None`; present quoted and unquoted values retain
their exact original content slices, including empty quoted content and
generic quoted whitespace. Name, equals, content and matching quote framing
must be genuinely present, in source order and inside the selected original
SourceBlock. Full attribute spans start at the name and end at its closing
quote, unquoted content or bare name. Original names and values are borrowed;
there is no entity decoder, expression parse, copied source or serialized
intermediate.

The real allocation-free Vue directive syntax provider must return plain
attribute syntax. Every directive, entity-bearing value and duplicate
authored name refuses. `class`, `style`, `key`, `ref` and `is` also remain
unavailable: normalization and Vue-specific owner/property semantics require
their own legalization. This matches the relevant distinctions in the
[pinned Vue parser](https://raw.githubusercontent.com/vuejs/core/v3.5.35/packages/compiler-core/src/parser.ts);
it does not use that parser as a production dependency.

The one existing canonical arena attribute vector constructor moves from the
body helper into the private header helper. The reviewed storage row records
that constructor and its two bound uses, without changing any storage policy
or budget. The unchanged consumption generator refreshes only actual L2
source counts; resolved legacy usage remains zero.

## Genuine laws and boundaries

The successful selected SFC law retains the same moved lower output, File,
original SourceBlock and canonical body operations. It pins five root
attributes in authored order, bare and empty values, quoted and unquoted
Unicode, real equals gaps, full UTF-8 spans, source-slice pointers and nested
input/span headers. Attributes do not mint canonical node ids; the actual
Element/Text body still has exactly four nodes.

A refusal at the last attribute retains the complete original header and
body, while the canonical File contains only an earlier completed root Text.
The root error stays sticky through later siblings and completion attempts.
A nested header refusal retains its completed parent attributes and actual
Text prefix but never mints the refused child Element. Equal-byte foreign
and reordered attribute-bearing roots cannot establish header custody.
Dropping, forgetting or unwinding the root after all headers and bodies complete still
denies a completed view and retains the same incomplete File.

Formatting, ordinary module discovery and reviewed storage gates run before
publication. Fresh exact-head Actions must compile and execute these laws;
protected queue full suites and unchanged instruction ceilings must pass
before actual merge. Previous provider or sibling fixture results grant no
execution credit to this new source.

Text entities and HTML whitespace, special attributes, directive operands,
handlers, If/For, SVG/MathML, components, other Vue dialects and native target
output remain unfinished. This grants no L4/SFC output, Vue runtime exposure,
default product migration or fix-history completion. #6838, #6839, #6840 and
the product/history gates remain open.
