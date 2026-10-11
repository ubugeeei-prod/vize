# Inline code remains inside its table cell

Issue: [#6101](https://github.com/ubugeeei-prod/vize/issues/6101).

The deployed Japanese JSX compatibility guide renders its original corpus path
36.671875 pixels into the neighboring role cell at a 1440-pixel viewport. The
existing table viewport check does not detect overlapping text inside a table.
The original source path must remain complete and readable.

Allow inline table code to wrap at whitespace or, for an uninterrupted path,
between characters. Preserve table scrolling, code outside tables, cell sizing,
all authored text, and every original browser check. The navigation observer
additionally captures every visible inline code text fragment and owning cell
rectangle, requiring the complete rendered text to stay inside that cell at the
existing one-pixel layout rounding tolerance. Exclude fenced blocks, which retain
their own scrolling contract. The first full Docs run additionally found existing
mobile architecture-table text drawn beyond its capped cell. Explicitly allow
normal whitespace wrapping on cells as well as inline code: inherited nowrap
otherwise places code's first fragment after the overflowing surrounding prose.
The original cell limits and table scrolling remain intact.

The original public page fails this geometry observer. Applying the two changed
CSS properties to that same page removes the overlap without changing its whole
content text. Fresh source Actions, full Docs rendering, protected merge, and
the deployed browser geometry remain delivery requirements. This slice does
not complete the documentation umbrella.
