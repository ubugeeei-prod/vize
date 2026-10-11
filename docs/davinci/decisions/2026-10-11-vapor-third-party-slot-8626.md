# Renamed and nested slot props (#8626)

Preserve Hebilicious's #8621 implementation and focused tests. Reusing the parsed
binding paths fixes renamed and nested reads; reverse override lookup preserves
the existing default-bearing read precedence.

The original whole SlotOwner and SlotConsumer SFCs, original issue text and
authored full expected DOM are registered in the legacy differential corpus.
The existing locked Vue VDOM oracle compares the complete rendered tree,
namespaces, events and disposal. The runtime maps the reporter's original child
import name without editing either SFC. Existing plain-key and lazy default
controls remain intact.

This layer follows the SVG layer to preserve shared test-module edits in one
linear native Stack. It does not claim a whole-product Davinci native path.
Actions, the protected queue and installed release validation must succeed
before closing the external report.
