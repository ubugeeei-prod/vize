# Docs syntax highlighting contrast

Tracked in [#8245](https://github.com/ubugeeei-prod/vize/issues/8245).

The public docs use a warm code palette. Comments have computed contrast ratios
of 2.856 in light mode and 3.248 in dark mode; several light syntax colors are
also faint. Slightly darken light-theme syntax colors and brighten muted dark
comments and punctuation. Retain the background, strong dark syntax colors,
base text, typography, highlighting grammar and original examples.

A preview of English/Japanese Vite+ guides and migration diffs in both themes
retains complete code text, HTML, token classes/order, fonts, bounds and scrolling
across 38 blocks and 356 tokens, including all 50 original signed diff lines.
Comment ratios rise to 3.943 and 4.571; light strings rise from 3.923 to 4.675,
and keywords from 4.430 to 5.061. No measured token contrast decreases. These
measurements describe this browser sample.

Keep the change in a small child PR based on the exact table-border parent and
register both in a native Stack. The parent can enter the queue as the ready
prefix. After its actual merge, refresh the child onto genuine current main,
retain every incoming canonical clause, and rerun exact-head Actions. Built and
public light/dark screenshot checks remain required after the preview.
