/**
 * Test-only fakes for camera capture: a controllable `MediaDevices` host,
 * tracks and streams, a recording 2D canvas, and object URLs.
 */
import type {
  WebcamMediaDeviceLike,
  WebcamMediaHost,
  WebcamMediaStreamLike,
} from "./webcam-capture-types.ts";

/** A fake track that records `stop()`. */
export class FakeTrack extends EventTarget {
  readonly kind: string;
  readyState = "live";
  stopped = false;

  constructor(kind = "video") {
    super();
    this.kind = kind;
  }

  stop(): void {
    this.stopped = true;
    this.readyState = "ended";
  }

  /** Simulate the device going away. */
  end(): void {
    this.readyState = "ended";
    this.dispatchEvent(new Event("ended"));
  }
}

/** A fake stream over fake tracks. */
export class FakeStream implements WebcamMediaStreamLike {
  readonly tracks: FakeTrack[];

  constructor(tracks: FakeTrack[] = [new FakeTrack()]) {
    this.tracks = tracks;
  }

  getTracks(): readonly FakeTrack[] {
    return this.tracks;
  }
}

interface PendingRequest {
  readonly constraints: MediaStreamConstraints | undefined;
  readonly resolve: (stream: WebcamMediaStreamLike) => void;
  readonly reject: (reason: unknown) => void;
}

/** A `MediaDevices` fake whose requests settle under test control. */
export class FakeHost extends EventTarget implements WebcamMediaHost {
  readonly requests: PendingRequest[] = [];
  devices: WebcamMediaDeviceLike[] = [];
  enumerations = 0;
  /** When set, requests settle immediately with this outcome. */
  auto: { readonly stream: WebcamMediaStreamLike } | { readonly error: unknown } | null = null;

  getUserMedia(constraints?: MediaStreamConstraints): Promise<WebcamMediaStreamLike> {
    return new Promise((resolve, reject) => {
      this.requests.push({ constraints, resolve, reject });
      if (this.auto === null) return;
      if ("stream" in this.auto) resolve(this.auto.stream);
      else reject(this.auto.error);
    });
  }

  enumerateDevices(): Promise<readonly WebcamMediaDeviceLike[]> {
    this.enumerations += 1;
    return Promise.resolve(this.devices);
  }
}

/** Create a `DOMException`-like error with a given name. */
export function mediaError(name: string): Error {
  const error = new Error(name);
  error.name = name;
  return error;
}

/** Calls recorded by the fake 2D context. */
export interface CanvasRecord {
  readonly calls: string[];
  readonly sizes: { width: number; height: number }[];
  readonly blobs: { type: string; quality: unknown }[];
  restore(): void;
}

/** Replace canvas 2D drawing and `toBlob` with recording fakes. */
export function installFakeCanvas(options: { readonly context?: boolean } = {}): CanvasRecord {
  const prototype = HTMLCanvasElement.prototype;
  const getContext = Object.getOwnPropertyDescriptor(prototype, "getContext");
  const toBlob = Object.getOwnPropertyDescriptor(prototype, "toBlob");
  const record: CanvasRecord = {
    calls: [],
    sizes: [],
    blobs: [],
    restore() {
      if (getContext) Object.defineProperty(prototype, "getContext", getContext);
      if (toBlob) Object.defineProperty(prototype, "toBlob", toBlob);
    },
  };
  Object.defineProperty(prototype, "getContext", {
    configurable: true,
    value(this: HTMLCanvasElement) {
      if (options.context === false) return null;
      record.sizes.push({ width: this.width, height: this.height });
      return {
        translate: (x: number, y: number) => record.calls.push(`translate(${x},${y})`),
        scale: (x: number, y: number) => record.calls.push(`scale(${x},${y})`),
        drawImage: (_source: unknown, ...args: number[]) =>
          record.calls.push(`drawImage(${args.join(",")})`),
      };
    },
  });
  Object.defineProperty(prototype, "toBlob", {
    configurable: true,
    value(callback: (blob: Blob | null) => void, type: string, quality: unknown) {
      record.blobs.push({ type, quality });
      callback(new Blob(["frame"], { type }));
    },
  });
  return record;
}

/** Give a video element intrinsic frame dimensions. */
export function setVideoSize(video: HTMLVideoElement, width: number, height: number): void {
  Object.defineProperty(video, "videoWidth", { configurable: true, value: width });
  Object.defineProperty(video, "videoHeight", { configurable: true, value: height });
}

/** Object-URL bookkeeping installed by {@link installFakeObjectUrls}. */
export interface ObjectUrlRecord {
  readonly created: string[];
  readonly revoked: string[];
  restore(): void;
}

/** Replace `URL.createObjectURL`/`revokeObjectURL` with counters. */
export function installFakeObjectUrls(): ObjectUrlRecord {
  const create = Object.getOwnPropertyDescriptor(URL, "createObjectURL");
  const revoke = Object.getOwnPropertyDescriptor(URL, "revokeObjectURL");
  const record: ObjectUrlRecord = {
    created: [],
    revoked: [],
    restore() {
      if (create) Object.defineProperty(URL, "createObjectURL", create);
      if (revoke) Object.defineProperty(URL, "revokeObjectURL", revoke);
    },
  };
  URL.createObjectURL = () => {
    const url = `blob:test/${record.created.length + 1}`;
    record.created.push(url);
    return url;
  };
  URL.revokeObjectURL = (url: string) => {
    record.revoked.push(url);
  };
  return record;
}

/** Let `srcObject` accept the structural fake streams used by these tests. */
export function installFakeSrcObject(): () => void {
  const prototype = HTMLMediaElement.prototype;
  const descriptor = Object.getOwnPropertyDescriptor(prototype, "srcObject");
  const values = new WeakMap<object, unknown>();
  Object.defineProperty(prototype, "srcObject", {
    configurable: true,
    get(this: HTMLMediaElement) {
      return values.get(this) ?? null;
    },
    set(this: HTMLMediaElement, value: unknown) {
      values.set(this, value);
    },
  });
  return () => {
    if (descriptor) Object.defineProperty(prototype, "srcObject", descriptor);
  };
}

/** Let pending promises and Vue updates settle. */
export async function flush(): Promise<void> {
  for (let index = 0; index < 4; index += 1) await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
}
