# First native application host

This private fixture compiles real Vue SFCs with Vize, delivers their view tree to
a native iOS adapter, and sends native button activations back to SFC state. Its
visible labels and buttons are native views. The execution web view is hidden
from accessibility; the UI tests cannot pass by finding web content.

The host is experimental and qualified only for the simulator. It provides text,
buttons, accessibility labels, and vertical stacks with bounded gap and padding.
It explicitly refuses unsupported capability requirements before mounting. The
negative fixture requests native images and verifies a native diagnostic.

## Run the complete fixture

Use a macOS host with Node 24, the repository Rust toolchain, Xcode, and an installed
iOS simulator runtime matching the selected simulator SDK. The app deployment
target is iOS 16. Use a clean tracked checkout, install the pinned workspace graph,
and run one qualification
command from the repository root:

```sh
vp install --filter './npm/native-host-fixture...'
vp run --filter './npm/native-host-fixture' qualify
```

Qualification verifies the SDK, builds the actual checkout compiler with its
existing producer receipt, builds Marquette, checks adapter types, compiles and
tests the SFC application, tests packet admission, and runs native simulator
interactions. The simulator used by that invocation is created separately and
removed afterward.

`06-native-application-host.yml` runs those checks on the repository macOS pool.
It retains the compiler receipt, application source hashes, emitted bundle,
native test logs, screenshots, and the native result bundle. The initial SDK
qualification proved that the selected Actions pool has the target SDK and a
matching installed runtime before this host integration was added.

## Source and engine boundary

`src/App.vue` and the three primitive `.vue` files are canonical authored sources
with explicit template, script setup, and style sections. Generated
`ios/Resources/application.js` is ignored and recreated by the compiler build.
No handwritten component render function is maintained as an application source.

The first host accepts typed primitive layout. Non-empty CSS and unsupported
native properties are refused. Its versioned view-tree boundary admits at most
1,024 nodes, depth 64, 4,096 UTF-16 units per text/label, and gap/padding from 0 through
4,096 points. Native node identifiers and update sequences are positive safe
integers. Duplicate, missing, cyclic, multiply owned, and unreachable nodes are
refused before replacing the visible tree. Native activations carry the accepted
snapshot sequence, so stale events cannot invoke another snapshot's handler.

The generated JS has a 64 KiB gzip budget; the simulator app has a 5 MiB binary
contribution budget. These are initial fixture limits. Simulator correctness and
size evidence do not qualify physical-device startup, interaction, memory, or
accessibility budgets.

Create templates, development refresh/clients, physical-device launch, complete
native UI families, native modules, OTA updates, signing, and distribution remain
open under #3132. This fixture does not close the managed or universal roadmaps.
