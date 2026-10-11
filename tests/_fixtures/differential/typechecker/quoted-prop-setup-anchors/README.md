# Quoted setup prop anchors: original Element Plus consumer

The two complete SFCs preserve Element Plus commit
`20298a21ae54a5417955373b80130ae7c548a6bb`. Their two complete original
projections preserve Vize producer
`605429c660b328bb03bec5bbfef15961e59fcaf0` and the actual native execution
from Actions run 38115632497. `original-custody.json` binds the upstream pin,
producer, provider archive, complete bytes and original native addresses.
`LICENSE` preserves the upstream MIT license.

The authored TypeScript parses. Both original projections contain
`void onUpdate:visible;` and fail parsing at that anchor. The differential
test regenerates each complete authored SFC and requires the complete new
projection to parse, preserving the authored property spelling and real
setup binding anchors. A separate control retains valid Unicode, dollar and
underscore identifiers plus quoted reserved, colon and hyphen property keys.

Run `cargo test --locked -p vize_canon --lib anchors::tests::`.
This fixture proves the generator repair; it does not claim the independent
native syntax-abort recovery or the original full consumer suite has passed.
