# Optional Vue peer admits 3.6 prereleases

Decision for [#7864](https://github.com/ubugeeei-prod/vize/issues/7864).

The published `vize` package's optional `vue: >=2.6.0` peer excludes all
prereleases under npm's ordinary SemVer rules. Preserve that stable floor and
add `>=3.6.0-0`, which admits prereleases of exactly the Vue 3.6.0 tuple.
Keep the peer optional; this does not change Vue dialect or runtime selection.
Future minor/major prereleases remain excluded unless separately admitted.

Retain the complete original report, including the installation command and
ERESOLVE transcript. The existing npm SemVer implementation checks complete
acceptance vectors for the original rc.10/beta.17, existing stable releases,
the lower floor, and unrelated prerelease tuples. No `includePrerelease`,
force install or legacy-peer-deps switch is used in the law.

Source Actions, protected merge and release remain required. This regression
law verifies source package metadata; an installed published-package check
remains a release task, and no successful npm installation is claimed here.
