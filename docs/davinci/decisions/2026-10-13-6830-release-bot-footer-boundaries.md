# Accept complete release bot footers at the end of a PR body

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The v0.448.0 metadata integration PR's catalog validation failed with
`Unterminated Codesmith decoration`. Its captured current body contained a
complete closing marker at the end of the body, without a final newline.
The current footer also omits the explanatory "Autofix is disabled" sentence
while retaining the explicit disabled marker. The settled CodeRabbit summary
uses Markdown dash bullets instead of asterisks.

Accept these observed formatting variants. A complete footer may end at EOF
or carry one final newline. Keep the complete generated policy, all four pins,
the repository and PR identity in both footer links, the disabled marker,
balanced envelopes, and the existing refusal of duplicate or unknown suffixes.
Bot summaries remain nonauthoritative and cannot relax qualification gates.

After #8778 merged, retain its reviewed stable-route driver. Authenticate the
entire composed driver before reversing exactly one cache-pin comment to that
known driver, then apply the existing whole stable-route inverse to the original
producer. Keep all nine original producer references and reject any unknown
body, reference pin, or path; this is no general source-hash exemption.

The original hosted event/current snapshots and the settled body are regression
fixtures under `tests/_fixtures/release/integration-body-448/`. Existing v0.447.0
fixtures and refusal tests remain in place. This correction fixes body parsing;
it does not qualify, merge, tag, or publish the failed v0.448.0 candidate.
