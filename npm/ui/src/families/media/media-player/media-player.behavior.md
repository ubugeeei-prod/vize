# MediaPlayer Behavior Contract

Normative state x input -> outcome table for the shared media controls
(`@vizejs/ui/media-player`): `media-player-root.vue`, `media-player-play-button.vue`,
`media-player-mute-button.vue`, `media-player-seek-slider.vue`,
`media-player-volume-slider.vue`, `media-player-time-display.vue`,
`media-player-playback-rate-button.vue`, `media-player-captions-button.vue`, and
`media-player-loading-indicator.vue`. The same parts ship as `VideoPlayer*` and
`AudioPlayer*`; the native element parts live in those families. Every row is
proven by the named test.

The native media element is the source of truth for playback. The root mirrors
its events (`play`, `pause`, `ended`, `waiting`, `seeking`, `timeupdate`,
`durationchange`, `progress`, `volumechange`, `ratechange`, `error`, text-track
changes, fullscreen, and picture-in-picture) into reactive state after mount.
Volume, muted state, and playback rate are controllable (`v-model:volume`,
`v-model:muted`, `v-model:playbackRate`); the element receives the resolved
values. Server rendering uses only props, so markup is deterministic.

| ID   | State                    | Input                                    | Outcome                                                                                                                                   | Evidence                                                                           |
| ---- | ------------------------ | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| MP1  | media registered         | render                                   | labelled `group` root, media id wiring, English labels, unseekable slider without a duration, disabled captions without caption tracks    | `renders labelled controls wired to the registered media element`                  |
| MP2  | paused / playing / ended | play button, native events               | toggles playback, labels follow Play/Pause/Replay, replay restarts, rejected `play()` emits `playRejected` and keeps `paused`             | `play, pause, and replay follow native events and report rejected playback`        |
| MP3  | known duration           | render / arrows / Page / Home / End      | `aria-valuetext` "1:23 of 5:00", progress and buffered CSS variables, ±`seekStep`, ±10 %, edges, clamped seeking, `update:currentTime`    | `the seek slider exposes time value text, keyboard seeking, and buffered progress` |
| MP4  | slider focused           | Arrow keys                               | sliders consume their keys, so root shortcuts do not seek twice                                                                           | `the seek slider exposes time value text, keyboard seeking, and buffered progress` |
| MP5  | playing                  | pointer scrubbing                        | seeks with pointer capture, pauses while scrubbing, ignores other pointers, resumes afterwards                                            | `pointer scrubbing seeks with capture and resumes playback afterwards`             |
| MP6  | any                      | volume keys / mute / native changes      | ±`volumeStep`, ±10 %, edges, rounding; mute toggles; control changes unmute; native changes sync without unmuting; controlled values wait | `volume and mute controls stay in sync with the element and controlled props`      |
| MP7  | RTL                      | volume pointer                           | the track maps from the reading-direction start                                                                                           | `volume pointer input maps the track width to the volume`                          |
| MP8  | any                      | rate button / native rate change         | cycles `rates`, restarts the cycle from unknown rates, mirrors native changes, rejects non-positive rates                                 | `playback rate cycles through rates and mirrors native rate changes`               |
| MP9  | caption tracks           | captions button / `setCaptionTrack`      | shows one caption/subtitle track, hides the rest, leaves other kinds untouched, and restores the last caption track                       | `captions toggle between hidden and the last or first caption track`               |
| MP10 | focus inside root        | Space/K, J/L, ←/→, ↑/↓, M, C, 0–9        | toggles play, skips ±`skipStep`, seeks ±`seekStep`, changes volume, mutes, toggles captions, seeks to percent; modified keys pass through | `root keyboard shortcuts control playback, time, volume, and captions`             |
| MP11 | editable / button focus  | shortcut keys                            | text fields keep their keys, Space activates the focused button once, and `keyboardShortcuts=false` disables shortcuts                    | `shortcuts ignore editable targets, focused buttons, and opt-out roots`            |
| MP12 | buffering / error        | `waiting`, `canplay`, `error`            | `data-loading` and the polite status report loading; errors emit `MediaError.code`                                                        | `loading and error states surface through data attributes, status, and emits`      |
| MP13 | localized                | `messages`, `ariaLabel`                  | every label and value text is localizable; `ariaLabel` overrides and `null` defers to slot text                                           | `messages localize every label and aria-label overrides are honored`               |
| MP14 | any                      | click with `preventDefault()` / no media | controls leave state unchanged; controls without media are disabled                                                                       | `controls honor preventDefault and stay disabled without media`                    |
| MP15 | `v-model:currentTime`    | parent value changes                     | the initial position applies on registration; parent values more than 0.5 s away seek                                                     | `synchronized currentTime seeks when the parent moves it`                          |
| MP16 | exposed instance         | imperative calls                         | exposes state and `play`, `pause`, `togglePlay`, `seek`, `seekBy`, volume, mute, captions, fullscreen, and picture-in-picture controls    | `exposes typed state and imperative controls`                                      |
| MP17 | missing provider         | setup                                    | every part fails closed with the shared context diagnostic                                                                                | `compound parts require a matching root provider`                                  |
| MP18 | helpers                  | pure functions                           | time formatting, durations, ranges, shortcuts, slider keys, editable targets, and message merging are deterministic                       | `media-player-format.test.ts`                                                      |
| MP19 | SSR                      | isolated requests                        | markup is byte-identical and reflects `defaultMuted`/`defaultVolume` without client state                                                 | `renders byte-identical media player markup across isolated SSR requests`          |
| MP20 | SSR / hydration          | hydrate                                  | server markup hydrates without warnings or node replacement, then registers the media element                                             | `hydrates media player markup without warnings or node replacement`                |
| MP21 | types                    | compile                                  | states, kinds, shortcuts, slot state, messages, and exposes are closed and read-only                                                      | `media-player.types.test-d.ts`                                                     |

Shortcut keys and listeners attach only on the client. Live streams report an
infinite duration and disable the seek slider.
