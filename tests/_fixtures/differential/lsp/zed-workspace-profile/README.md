# Zed workspace initialization preserves project choices

The two TypeScript configs retain the complete code blocks from #8007 and
#7196. Neither report supplies a complete SFC; `Profile.vue.txt` is an authored
whole-source control with an exact declaration hover. The real-server launcher
copies each config unchanged to `vize.config.ts` in a real Vue/TypeScript
workspace and sends the production extension selector's actual result.

The six cases cover project-owned defaults, a partial formatting override,
explicit empty options, an intentional explicit recommended profile, the
no-config recommended default, and the original #7196 project choices.
Complete hover replies and the relevant advertised capabilities are exact.
The current server omits disabled capability keys: absence is asserted separately
from their JSON value, so an explicit null or false does not substitute for it.
The existing shared completion/diagnostics/formatting/rename contract remains
intact. Runtime discovery is provided by a `node_modules/.bin/tsgo` link, not
by editing the reporter's config or adding initialization feature switches.

Zed's opaque `Worktree` cannot be constructed by the native scenario. The
extension callback uses the pinned SDK's root-relative `read_text_file`, while
the scenario exercises the exact shared selector with real filesystem reads
and the actual Vize stdio process. The official pinned extension CLI compiles
and validates the WASM extension separately. This is not a Zed GUI-host claim.

Run `vp run --workspace-root test:zed-extension:real-server` on Actions with
the source-built CLI and real Vue/TypeScript dependencies. Native extension
unit controls cover all five discovered config names and untouched explicit
values without invoking worktree reads. No server or native-stage contract is
changed by this editor-only fix.
