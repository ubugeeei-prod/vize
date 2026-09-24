/** Compile-only assertions for the `use-user-media` type contracts. */

import { useMediaStream, useUserMedia } from "./use-user-media.ts";
import type {
  MediaStreamErrorCode,
  MediaStreamLike,
  MediaStreamStatus,
  UserMediaHost,
} from "./use-user-media.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

const camera = useUserMedia({ constraints: { video: { facingMode: "user" } } });
type _StatusIsClosed = Expect<Equal<typeof camera.status.value, MediaStreamStatus>>;
type _ErrorCodeIsClosed = Expect<
  Equal<NonNullable<typeof camera.error.value>["code"], MediaStreamErrorCode>
>;
type _StartResolvesStream = Expect<
  Equal<Awaited<ReturnType<typeof camera.start>>, MediaStreamLike | undefined>
>;

declare const devices: MediaDevices;
devices satisfies UserMediaHost;
declare const nativeStream: MediaStream;
nativeStream satisfies MediaStreamLike;

useMediaStream({
  source: { request: async (_fps: number) => nativeStream },
  constraints: 30,
});

useMediaStream({
  source: { request: async (_fps: number) => nativeStream },
  // @ts-expect-error constraints must match the source.
  constraints: "30",
});

// @ts-expect-error the status is read-only.
camera.status.value = "active";
