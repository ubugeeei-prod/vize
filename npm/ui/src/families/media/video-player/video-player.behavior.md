# VideoPlayer Behavior Contract

Normative state x input -> outcome table for `video-player-video.vue`,
`video-player-fullscreen-button.vue`, and `video-player-picture-in-picture-button.vue`
(`@vizejs/ui/video-player`). The shared controls re-exported as `VideoPlayer*`
follow `media-player.behavior.md`. Every row is proven by the named test.

| ID   | State                | Input                         | Outcome                                                                                                                            | Evidence                                                                        |
| ---- | -------------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| VP1  | render               | props                         | native `<video>` with validated `src`/`poster`, `playsinline`, preload, CORS, size, slot `<track>`s; unsafe sources are dropped    | `renders a native video with validated sources and inline-playback attributes`  |
| VP2  | `defaultMuted`       | autoplay                      | the element renders and stays muted so muted autoplay is allowed                                                                   | `defaultMuted mutes the element for muted autoplay`                             |
| VP3  | fullscreen supported | button / F key                | the root container enters and leaves fullscreen; state, `data-fullscreen`, and labels follow `fullscreenchange`                    | `fullscreen toggles the root through the standard API and reports state`        |
| VP4  | prefixed platforms   | toggle                        | falls back to `webkitRequestFullscreen`, then native iOS video fullscreen with `webkitbeginfullscreen`/`webkitendfullscreen`       | `fullscreen falls back to WebKit prefixes and native video fullscreen`          |
| VP5  | PiP supported        | button                        | enters and leaves picture-in-picture, tracking `enterpictureinpicture`/`leavepictureinpicture`; `disablePictureInPicture` opts out | `picture-in-picture is detected after mount and toggles the video`              |
| VP6  | unsupported          | render                        | platform buttons are disabled, or hidden with `unsupported="hide"`, and report `data-supported="false"`                            | `unsupported platform controls are disabled or hidden`                          |
| VP7  | any                  | click with `preventDefault()` | the request is skipped; `ariaLabel` overrides the default label                                                                    | `platform buttons honor preventDefault and aria-label overrides`                |
| VP8  | aliases              | import / setup                | `VideoPlayer` aliases the shared root; video parts require the provider                                                            | `VideoPlayer aliases the shared root and requires a provider for its parts`     |
| VP9  | SSR                  | isolated requests             | markup is byte-identical and platform buttons render the unsupported state until mount                                             | `renders byte-identical video player markup with unsupported platform controls` |
| VP10 | SSR / hydration      | hydrate                       | server markup hydrates without warnings or node replacement                                                                        | `hydrates video player markup without warnings or node replacement`             |
| VP11 | types                | compile                       | the root is the shared MediaPlayer root and unsupported behavior is a closed union                                                 | `video-player.types.test-d.ts`                                                  |

Platform capabilities are detected after mount. WebKit-prefixed APIs are reached
through `in` checks and `Reflect`, never through casts.
