import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import { useBattery } from "./battery.ts";
import { useDeviceMotion } from "./device-motion.ts";
import { useDeviceOrientation } from "./device-orientation.ts";
import { useDevicePixelRatio } from "./device-pixel-ratio.ts";
import { useFps } from "./fps.ts";
import { useGeolocation } from "./geolocation.ts";
import { useIdle } from "./idle.ts";
import { useNetwork, useOnline } from "./network.ts";
import { usePageLeave } from "./page-leave.ts";
import {
  usePreferredColorScheme,
  usePreferredContrast,
  usePreferredDark,
  usePreferredReducedTransparency,
} from "./preferences.ts";
import { usePreferredLanguages } from "./preferred-languages.ts";
import { useScreenOrientation } from "./screen-orientation.ts";

const Probe = defineComponent({
  setup() {
    const geo = useGeolocation();
    const network = useNetwork();
    const online = useOnline();
    const battery = useBattery();
    const orientation = useDeviceOrientation();
    const motion = useDeviceMotion();
    const screen = useScreenOrientation();
    const idle = useIdle();
    const left = usePageLeave();
    const dark = usePreferredDark();
    const scheme = usePreferredColorScheme();
    const contrast = usePreferredContrast();
    const transparency = usePreferredReducedTransparency();
    const languages = usePreferredLanguages({ ssrLanguages: ["ja", "en"] });
    const ratio = useDevicePixelRatio();
    const fps = useFps();

    return () =>
      h(
        "pre",
        JSON.stringify({
          geo: [geo.isSupported.value, geo.coords.value, geo.capability.value.status],
          network: [
            network.isSupported.value,
            network.isOnline.value,
            online.value,
            network.type.value,
          ],
          battery: [battery.isSupported.value, battery.level.value, battery.charging.value],
          orientation: [
            orientation.isSupported.value,
            orientation.alpha.value,
            orientation.permission.value,
          ],
          motion: [motion.isSupported.value, motion.acceleration.value],
          screen: [screen.isSupported.value, screen.orientation.value, screen.angle.value],
          idle: [idle.idle.value, idle.lastActive.value],
          left: left.value,
          preferences: [dark.value, scheme.value, contrast.value, transparency.value],
          languages: languages.value,
          ratio: [ratio.pixelRatio.value, ratio.isSupported.value],
          fps: [fps.fps.value, fps.isSupported.value],
        }),
      );
  },
});

void test("device sensors render deterministic server values without browser globals", async () => {
  assert.equal(typeof globalThis.window, "undefined");
  const first = await renderToString(createSSRApp(Probe));
  const second = await renderToString(createSSRApp(Probe));
  assert.equal(first, second);

  const payload: unknown = JSON.parse(
    first
      .replace(/^<pre>/, "")
      .replace(/<\/pre>$/, "")
      .replaceAll("&quot;", '"'),
  );
  assert.deepEqual(payload, {
    geo: [false, null, "unavailable"],
    network: [false, true, true, null],
    battery: [false, 1, true],
    orientation: [false, null, "prompt"],
    motion: [false, null],
    screen: [false, "portrait-primary", 0],
    idle: [false, null],
    left: false,
    preferences: [false, "no-preference", "no-preference", false],
    languages: ["ja", "en"],
    ratio: [1, false],
    fps: [0, false],
  });
});
