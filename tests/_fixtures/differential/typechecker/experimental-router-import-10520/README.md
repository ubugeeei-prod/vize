This active batch case is the actual imported `definePage` false negative from
#10520. Its complete generated route declaration and copied provider authority
are pinned together with the invalid page and opt-in config. The named batch
execution hydrates the shared real 5.4.0 provider fixture, compares its authority
byte for byte, runs its actual scanner, then overlays the unchanged invalid page
without regenerating declarations. It compares every returned diagnostic.

The dedicated `router-experimental-check.test.ts` is the wider CLI execution
witness: complete program reports and official reference output across valid,
invalid, and repaired LF/CRLF sources. This case does not fabricate router types.
