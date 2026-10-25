# Component input diagnostic ownership

These minimized authored inputs independently reproduce #10607 and #10618 with
published `vize@0.566.0`, Vue 3.5.42 and the physical published TypeScript
7.1.0-dev.20261024.1 host. The generic form keeps three variants, generic schema
inference, imported slot types and both events. The small Child3/Min pair is the
reported named-slot reproduction.

The mapper must retain one complete diagnostic at the authored binding while
preserving callback, event, slot-body and fallback checks. Tests retain complete
host/CLI/protocol observations and validate LF/CRLF, Unicode, deferred owner
buffers and independent files with identical source offsets.
