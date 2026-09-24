/** Compile-only assertions for the device and preference sensor composables. */

import type { ComputedRef, Ref, ShallowRef } from "vue";

import { useBattery } from "./battery.js";
import type { BatteryHost } from "./battery.js";
import type { CapabilityResult } from "./capability.js";
import { useDeviceMotion } from "./device-motion.js";
import type { MotionVector } from "./device-motion.js";
import { requestMotionPermission, useDeviceOrientation } from "./device-orientation.js";
import type { MotionPermissionState } from "./device-orientation.js";
import { useDevicePixelRatio } from "./device-pixel-ratio.js";
import { useFps } from "./fps.js";
import { useGeolocation } from "./geolocation.js";
import type {
  GeolocationCoordinatesSnapshot,
  GeolocationUnavailableReason,
} from "./geolocation.js";
import { useIdle } from "./idle.js";
import { useNetwork, useOnline } from "./network.js";
import type { NetworkConnectionType, NetworkEffectiveType } from "./network.js";
import { usePageLeave } from "./page-leave.js";
import { usePreferredColorScheme, usePreferredContrast } from "./preferences.js";
import type { ColorSchemePreference, ContrastPreference } from "./preferences.js";
import { usePreferredLanguages } from "./preferred-languages.js";
import { useScreenOrientation } from "./screen-orientation.js";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const geo = useGeolocation();
type _GeoCapability = Expect<
  Equal<
    typeof geo.capability,
    ComputedRef<CapabilityResult<GeolocationCoordinatesSnapshot, GeolocationUnavailableReason>>
  >
>;
type _GeoCoordsAreShallowSnapshots = Expect<
  Equal<typeof geo.coords, Readonly<ShallowRef<GeolocationCoordinatesSnapshot | null>>>
>;
// @ts-expect-error watch options are numeric milliseconds.
useGeolocation({ timeout: "5s" });

const network = useNetwork();
type _EffectiveTypeIsClosed = Expect<
  Equal<typeof network.effectiveType, Readonly<Ref<NetworkEffectiveType | null>>>
>;
type _ConnectionTypeIsClosed = Expect<
  Equal<typeof network.type, Readonly<Ref<NetworkConnectionType | null>>>
>;
type _OnlineIsBoolean = Expect<Equal<ReturnType<typeof useOnline>, Readonly<Ref<boolean>>>>;

const battery = useBattery();
battery.level.value satisfies number;
// @ts-expect-error battery refs are readonly.
battery.level.value = 0;
// @ts-expect-error getBattery must return a promise.
const _syncBattery: BatteryHost = { getBattery: () => undefined };

const orientation = useDeviceOrientation();
type _PermissionIsClosed = Expect<
  Equal<ReturnType<typeof orientation.requestPermission>, Promise<MotionPermissionState>>
>;
type _RequestPermissionHelper = Expect<
  Equal<ReturnType<typeof requestMotionPermission>, Promise<MotionPermissionState>>
>;
const motion = useDeviceMotion();
type _MotionSnapshot = Expect<
  Equal<typeof motion.acceleration, Readonly<ShallowRef<MotionVector | null>>>
>;

const screen = useScreenOrientation();
screen.orientation.value satisfies OrientationType;
// @ts-expect-error lock targets are the platform union.
void screen.lockOrientation("upside-down");

const idle = useIdle(1000);
type _LastActiveNullable = Expect<Equal<typeof idle.lastActive, Readonly<Ref<number | null>>>>;
// @ts-expect-error the timeout is numeric.
useIdle("1m");

usePageLeave().value satisfies boolean;

type _SchemeIsClosed = Expect<
  Equal<ReturnType<typeof usePreferredColorScheme>, ComputedRef<ColorSchemePreference>>
>;
type _ContrastIsClosed = Expect<
  Equal<ReturnType<typeof usePreferredContrast>, ComputedRef<ContrastPreference>>
>;
// @ts-expect-error server preferences are closed unions.
usePreferredColorScheme({ ssrPreference: "sepia" });

type _LanguagesAreReadonly = Expect<
  Equal<ReturnType<typeof usePreferredLanguages>, Readonly<Ref<readonly string[]>>>
>;

useDevicePixelRatio().pixelRatio.value satisfies number;
const fps = useFps({ every: 30 });
type _FpsIsReadonly = Expect<Equal<typeof fps.fps, Readonly<Ref<number>>>>;
