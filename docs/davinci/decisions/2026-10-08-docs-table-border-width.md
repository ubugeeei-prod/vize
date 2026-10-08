# Docs table border width

Tracked in [#8240](https://github.com/ubugeeei-prod/vize/issues/8240).

The public Crate Reference shows a right-hand outline beyond the actual table
columns. At a 1440 × 960 viewport, the Distribution Layers outline spans 876 px
while its row spans 488.156 px, leaving 386.844 px of empty space on the right.
The navigation theme makes the table a block scroll container; its inherited
full width and outer border outline the article width instead of the column grid.

Set `width: fit-content` in the existing `.content table` rule. Keep its current
border, maximum width, column widths, wrapping rules and horizontal overflow.
The table outline then follows the intrinsic grid on desktop and remains bounded
by the article on narrower screens.

The browser preview uses the actual public Crate Reference at 1440 × 960 and
390 × 844. All five desktop tables have exactly the existing 1 px border beside
their grids. Across all ten cases, the complete table markup, measured column
widths, wrapping and heights are unchanged. All five mobile tables preserve their
scroll widths and actually scroll to the same horizontal endpoints without
overflowing the document.

Delivery requires exact-head Actions, the protected merge and fresh public
desktop/mobile CSS and screenshot verification. The browser-injected preview
qualifies the proposed layout; built and deployed delivery remain pending.
