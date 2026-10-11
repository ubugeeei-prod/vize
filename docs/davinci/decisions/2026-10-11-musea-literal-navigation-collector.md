# Literal variant navigation collector

Issue: [#8534](https://github.com/ubugeeei-prod/vize/issues/8534).

The original native static and hosted collectors waited for any current table-of-contents
item before asserting that the first item was current. Intersect the original first-item
locator with the current-attribute locator and retain the original default wait, clicks,
final assertion, smooth scrolling, and product observer behavior.

Retain bounded requested/current names, aria attributes, section positions, scroll state,
and wall and monotonic timestamps before navigation and after synchronization, including
failure. Observation errors remain in the existing artifact without masking the original
wait or assertion error. Producer IntersectionObserver events are unavailable; these DOM
snapshots are not callback evidence, and the observed ordering cause remains unknown.

Timer-free locator-contract controls preserve pending first-target selection and upstream
error and deadline refusal. They do not establish native runtime success or stability.
Preserve both original protected failures and qualify the combined collector successor
with the actual native browser controls before claiming that it passes.
