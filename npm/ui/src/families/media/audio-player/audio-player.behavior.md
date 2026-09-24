# AudioPlayer Behavior Contract

Normative state x input -> outcome table for `audio-player-audio.vue`
(`@vizejs/ui/audio-player`). The shared controls re-exported as `AudioPlayer*`
follow `media-player.behavior.md`. Every row is proven by the named test.

| ID  | State           | Input             | Outcome                                                                                                   | Evidence                                                                  |
| --- | --------------- | ----------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| AP1 | render          | props             | native `<audio>` with validated `src`, preload, loop, controls, and slot `<source>`s, registered as audio | `renders a native audio element registered as audio media`                |
| AP2 | registered      | controls          | slot-labelled play controls and time displays drive the element                                           | `slot-labelled controls drive audio playback and time display`            |
| AP3 | unsafe source   | render            | the source is dropped and marked with `data-invalid-src`                                                  | `unsafe audio sources are dropped and marked`                             |
| AP4 | registered      | part unmounts     | the root unregisters the element and clears `data-media-kind`                                             | `unmounting the audio part unregisters it from the root`                  |
| AP5 | SSR             | isolated requests | markup is byte-identical                                                                                  | `renders byte-identical audio player markup across isolated SSR requests` |
| AP6 | SSR / hydration | hydrate           | server markup hydrates without warnings or node replacement                                               | `hydrates audio player markup without warnings or node replacement`       |
| AP7 | types           | compile           | the root and controls are the shared MediaPlayer parts; no video part is exported                         | `audio-player.types.test-d.ts`                                            |
