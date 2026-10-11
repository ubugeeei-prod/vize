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
