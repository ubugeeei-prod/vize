# Installed public Musea failure observations

Issue: [#8463](https://github.com/ubugeeei-prod/vize/issues/8463).

The genuine installed-public 0.439 replay [38032119383](https://github.com/ubugeeei-prod/vize/actions/runs/38032119383) reaches Musea and fails the original first-frame predicate, but the old harness preserves only its timeout. Diagnosis requires the actual component Documents, globals/setup state, script errors and HTTP modules. A separate published consumer demonstrates why: initial globals and setup are correct while inline Self renders an unresolved element ([#8460](https://github.com/ubugeeei-prod/vize/issues/8460)).

## Decision

Add a consumer-owned browser observer, copied byte-exactly during preparation and included in harness custody verification. Observe page/console/request errors and browser responses; retain whole text/module/JSON response bytes with URL/content SHA256 identities. Preserve active iframe HTML, actual initial globals/setup tokens, native button/unresolved-element counts and screenshots on failure. Response resource types distinguish browser script loading from fetch observations; browser-provided bytes are reported exactly, including an empty body where Chromium did not retain one. This makes no claim that an unretained browser body contains the server's response bytes.

The helper records no success receipt. The original error is rethrown even if artifact capture fails. Every existing readiness predicate and its 30-second deadline, globals/URL/non-remount/trust/legacy assertion, override refusal, Linux provider custody, independent native/browser workflow condition and strict final seal remains unchanged. Capture applies to the gallery, its popups, the clean copied-URL context and legacy one-argument setup case.

## Evidence and boundary

A real HTTP/Chromium negative control calls the original first-frame function with two unresolved preview elements and waits for its unchanged deadline. It requires both actual iframe HTML documents and hashes, correct initial globals/setup tokens, absent native buttons, actual script errors/console output, genuine HTTP 404 browser script/fetch responses, retained whole fetched response bytes and a screenshot. It refuses a success receipt. Existing public authority and retirement laws remain part of source Actions.

Replaying the unchanged original browser predicates against the genuine installed 0.439 packages with this observer retains seven actual Documents and their module responses, exposing the component mismatch without a success receipt. That local macOS diagnostic is not official Linux acceptance; the final proof still requires the independently installed published release on reviewed main after terminal publication.
