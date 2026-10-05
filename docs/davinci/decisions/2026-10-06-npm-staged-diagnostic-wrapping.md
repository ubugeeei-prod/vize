# Wrapped npm staged-version diagnostics

Paired issue: [#8054](https://github.com/ubugeeei-prod/vize/issues/8054).

The native npm publish job on literal release candidate
`8a8521d6897bbe3fd0af0cbfaebd83f4fc933933` returned a rendered409 diagnostic
that wraps between `staged` and `version`. The existing contiguous phrase check
missed it, so all six ordinary publish attempts ran. The complete five-line
stderr payload is retained unchanged in
`tests/_fixtures/tooling/npm-publish-staged/wrapped-409.stderr.txt`; its source
record pins job111812350029, run37318076195 attempt2, original log lines and
the full job-log SHA256. Registry visibility causes remain unknown.

Decision: fold only ASCII spaces, tabs, CR/LF and the rendered `│` border in
the private diagnostic recognition view. Require the same complete
`Cannot publish over previously staged version` phrase. Continue to print
the untouched stdout/stderr and run the same publish command. A recognized
staged conflict enters the existing bounded registry wait, requiring both the
exact version and its requested tag; polling limits and failure policy stay
unchanged. Generic409 and authorization errors retain ordinary retries.

The existing staged-helper test remains byte exact. Eight new complete helper
cases use the original LF diagnostic, CRLF, narrower wrapping, stdout,
unrelated409 and authorization errors, a hidden version and a wrong tag.
They execute the actual Moon helper against the existing fake-command test
boundary, verify whole stderr, publisher arguments/cwd, invocation count and
bounded version/tag probes. This is tooling behavior qualification, with no
registry publication or product runtime acceptance implied.

Validation is pending source Actions and independent technical review.
Keep the conventional PR Draft with auto-merge and queue entries absent until
the maintainer ends the first0.433 publication hold. The current immutable
release tool/tag/candidate is untouched. Actual protected merge and inclusion
in the next published release remain required.
