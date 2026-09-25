# WebcamCapture Behavior Contract

Normative state x input -> outcome table for `webcam-capture-root.vue`,
`webcam-capture-video.vue`, `webcam-capture-start-button.vue`,
`webcam-capture-stop-button.vue`, `webcam-capture-switch-camera.vue`,
`webcam-capture-device-select.vue`, `webcam-capture-shutter.vue`,
`webcam-capture-photo.vue`, and `webcam-capture-status-message.vue`
(`@vizejs/ui/webcam-capture`). Every row is proven by the named test.

The root either owns a camera stream (acquired through `getUserMedia` only after
`start()` or `autoStart` on mount) or displays a consumer stream passed through
`stream`, such as `useUserMedia().stream` from `@vizejs/composable`. Status values
and error codes are identical to the composable's `MediaStreamStatus` and
`MediaStreamErrorCode`. Consumer streams are never acquired, restarted, or stopped.

| ID  | State             | Input                               | Outcome                                                                                                                                    | Evidence                                                                         |
| --- | ----------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| W1  | idle              | mount                               | renders idle parts with localized defaults, a mirrored muted inline preview, a polite live region, and never requests permission           | `renders an idle camera without requesting permission`                           |
| W2  | idle              | Start                               | requests `{ audio, video: { ...constraints, facingMode } }`, announces requesting/active, binds `srcObject`, and lists video inputs        | `start acquires the camera, binds the preview, and lists devices`                |
| W3  | requesting        | rejection / missing API             | maps errors with the composable table (`permission-denied`, `unsupported`, …), emits `error`, announces the localized reason, allows retry | `acquisition failures are normalized like useUserMedia`                          |
| W4  | active (owned)    | Stop / all tracks end / late stream | stops owned tracks, unbinds the preview, returns to idle; a stream resolved after `stop` is released immediately                           | `stop and ended tracks release owned streams, and late streams are discarded`    |
| W5  | active (owned)    | Switch camera / device select       | toggles facing mode (mirroring follows `user`) or selects an exact device, emits `update:*`, and restarts the owned stream                 | `switching cameras and selecting devices restart an owned stream`                |
| W6  | external stream   | render / start / stop / unmount     | shows the consumer stream as active, disables owned controls, never calls `getUserMedia`, never stops consumer tracks                      | `external streams are displayed but never acquired or stopped`                   |
| W7  | active            | Shutter                             | draws a mirrored, center-cropped, downscaled frame, encodes with `captureType`/`captureQuality`, renders the photo, revokes old URLs       | `the shutter captures a mirrored, cropped frame and owns the photo URL`          |
| W8  | idle / no frame   | capture                             | captures nothing while idle; draw and encode failures emit `captureError` with typed diagnostics                                           | `capture failures emit captureError and a missing frame captures nothing`        |
| W9  | active            | Shutter with `countdown`            | announces each remaining second, disables the shutter, captures after the countdown; `stop` cancels it                                     | `countdowns announce each second, block the shutter, and cancel on stop`         |
| W10 | any               | `messages`                          | every default label and announcement is replaced by the typed `messages` prop                                                              | `messages localize every default label and announcement`                         |
| W11 | `autoStart`       | mount / `devicechange`              | requests on mount (client only) and refreshes the device list on `devicechange`; `mirrored` overrides the facing-mode default              | `autoStart requests on mount and devicechange refreshes the list`                |
| W12 | active with photo | unmount                             | stops owned tracks and revokes the photo object URL                                                                                        | `unmounting stops owned tracks and revokes the photo URL`                        |
| W13 | controls          | click with `preventDefault()`       | leaves the camera untouched                                                                                                                | `buttons honor preventDefault from click listeners`                              |
| W14 | missing provider  | setup                               | parts fail closed with the shared context diagnostic                                                                                       | `compound parts require a matching root provider`                                |
| W15 | helpers           | pure functions                      | error normalization, constraints, device lists, message merging, and capture geometry are deterministic                                    | `webcam-capture-media.test.ts`                                                   |
| W16 | SSR               | isolated requests                   | idle markup is byte-identical and `autoStart` never touches devices on the server                                                          | `renders byte-identical idle markup without requesting the camera on the server` |
| W17 | SSR / hydration   | hydrate                             | server markup hydrates without warnings or node replacement                                                                                | `hydrates idle camera markup without warnings or node replacement`               |
| W18 | types             | compile                             | statuses and codes match the composable, native `MediaStream`/`MediaDevices` are structurally accepted, options are closed                 | `webcam-capture.types.test-d.ts`                                                 |

Mirroring is published as `data-mirrored` and `--vize-ui-webcam-capture-scale-x`
(`-1` or `1`); apply it with `transform: scaleX(var(--vize-ui-webcam-capture-scale-x))`.
Browsers only accept native `MediaStream`s as `srcObject`; the root binds the raw
(unproxied) stream and skips structural stand-ins.
