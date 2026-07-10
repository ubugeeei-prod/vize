# vize_rendu

Rendu is Vize's borrowed render projection. It gives emitters, inspectors, and
profilers a common output-facing vocabulary without owning the source AST or
semantic analysis.

- Relief owns source syntax and locations.
- Croquis owns derived meaning and relationships.
- Rendu projects the transformed syntax into render operations and output
  sections.
- Atelier crates consume Rendu to emit a concrete target.

Rendu views borrow existing compiler buffers. The abstraction is intended to
be allocation-free; it does not imply that compilation or generated runtime
code has zero cost.
