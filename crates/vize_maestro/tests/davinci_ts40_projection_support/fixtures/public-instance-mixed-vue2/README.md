# Public-instance reads in the mixed Vue 2 fixture

This retains every complete before/after packet for `vue2-native-event`, plus
the exact rendered projection records, before advancing its legacy-enabled
TS-40 snapshot for #10933. `.ts` and `.vue` packets have a `.txt` suffix for
storage only. No packet bytes are formatted or regenerated.

The before source is `19828f974f80e857f74566078947fa5981c6b26b`, captured by
[run 40484190377](https://github.com/ubugeeei-prod/vize/actions/runs/40484190377),
[legacy artifact 12724606040](https://github.com/ubugeeei-prod/vize/actions/runs/40484190377/artifacts/12724606040).
Artifact `12724177839` from that run is the default-feature capture and cannot
supply the enabled legacy packets. The before Canon, Croquis, Maestro, TS-40
fixture trees and Cargo.lock equal PR base
`481c94bd55a7276339adc277edd93e30313f6ea9`; Git object IDs are in the manifest.

The after source is `45ac0a89859360f45f466b8f1178d15ac26be286`, captured by
[run 40494413893](https://github.com/ubugeeei-prod/vize/actions/runs/40494413893),
[legacy artifact 12730067601](https://github.com/ubugeeei-prod/vize/actions/runs/40494413893/artifacts/12730067601).
The manifest authenticates both original ZIPs and the length/SHA-256 census of
all 205 packet files across the matrix. Exactly eight files change: the four
previously reviewed `options-api-slots` Maestro packets and four
`vue2-native-event` Maestro packets. All 36 retained native-fixture packets and
rendered records have individual length/SHA-256 witnesses.

The fixture is deliberately mixed. `capture_canon` passes its
`options_api=true, legacy_vue2=true` settings to the document generator.
`capture_maestro` calls `VirtualCodeGenerator::new().generate()` without those
settings. The latter's checker document uses
`analyze_sfc_descriptor_with_context` and
`generate_virtual_ts_with_split_offsets`, whose generation options keep the
default `legacy_vue2=false` and Vue 3 dialect. Enabling the Cargo `legacy`
feature permits this fixture's capture; it does not change those settings.
The relevant harness, entry-point and generation-option Git objects are
unchanged across the two captures and recorded in the manifest.

Consequently the real Canon Vue 2 template, its mappings, semantic links,
import map and every byte-offset import probe remain byte-identical. The
fixed Vue 3 Content Mapper, original SFC and Maestro script also remain
byte-identical. The changed Maestro document already used Vue 3's
`import('vue').ComponentPublicInstance` context before this PR. Its new
public-instance handling adds 494 bytes only in the undefined-reference block:
`message` gets a typed local and one public-instance member read at each
authored occurrence (`160:167` and `173:180`). The old explicit free-name read
mapped only `160:167`. The first mapping moves from `16473:16480` to
`16887:16894`; the second is `16967:16974`. Every old authored span and feature
flag remains, with one extra mapping (8 to 9) and one extra hit (6 to 7).

Only the Maestro text, mapping and authored-hit record fields advance. The
before record is frozen alongside the complete before packets. The evidence
test authenticates both packet sets under every feature configuration and,
with `legacy` enabled, compares the live complete mixed record to the retained
after record. This records the existing harness routing and the reviewed
default-generator transition; it does not claim Vue 2 diagnostic parity for
Maestro's default Vue 3 route. Actual legacy Canon generation is unchanged in
this fixture.
