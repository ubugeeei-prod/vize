# Testing and feedback reading flow

Issue: [#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

The English testing guide opened with the project's testing infrastructure;
the Japanese guide still contained broken machine-translated sentences.
Both now help a reader inspect one affected Vue file, share a reproduction,
and explain the observed and expected behavior before discussing test sources.
Performance reports and offering a licensed project remain separate next steps.

This slice owns only `guide/testing.md` in English and Japanese and its existing
More guides coverage entry. The original Inspector, report, test-case,
performance, and project-fixture headings remain. Existing commands, fixture
paths, project examples, and the actual external `SUPPORT.md` URL remain.
The Japanese performance link explicitly identifies the existing English guide.

Preparation starts from the complete Japanese documentation parent
`4ec3cfd32245499bdf2f1d912986c56d98157e32`. Its source and published PRs remain
unchanged. Existing configuration, reference examples, 31 Japanese paragraph
witnesses, rendering checks, and the full navigation route factory are retained.
Marking this page revised adds its English and Japanese routes to the existing
More guides renderer without removing any previously captured route.

Source review covers both complete testing guides. Their new rendered desktop
and mobile reading flow and deployed links still require verification; evidence
from the parent does not qualify this change. Other More guides requirements
remain open under #8365. No new test framework or runtime behavior is introduced.

## Published Japanese reading flow

#8629 actually merged as `5390fc5d3ffa84fe4ffcf0e71f855174203d1279`.
Official source `130331f713bdfa1db41abce506e0f2ce2ee5486f` contains that
merge and passed full Docs `38151503446`; successful publisher `38155628417`
and deployment `6998412044` supplied its public site. Native Chrome at 1440 and
390 pixels retained all six authored Japanese headings and six inline syntax
values, with no page overflow, and actually followed the Inspector guidance to
the Japanese compiler-inspector page. The first mobile view was visually read.
The [public receipt](./results/japanese-public-reader-130.json) records complete
heading/source hashes, native destinations and PNG hashes; the exact public
manifest remained unchanged across all six contribution/testing/credits cases.
This completes the bounded Japanese public flow, while the complete More guide
review and other site-wide requirements remain open.
