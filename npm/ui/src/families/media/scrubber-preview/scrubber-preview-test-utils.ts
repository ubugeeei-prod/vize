/**
 * Test-only video, canvas, and object-URL fakes. happy-dom neither decodes
 * video nor rasterizes canvases, so these model the observable contract:
 * metadata readiness, `seeked` after a `currentTime` write, frame drawing, and
 * blob encoding.
 */

/** Controls for the installed fakes. */
export interface FakeCaptureEnvironment {
  /** Object URLs created, in order. */
  readonly created: string[];

  /** Object URLs revoked, in order. */
  readonly revoked: string[];

  /** Times drawn to a canvas, in order. */
  readonly drawn: number[];

  /** Pending seeks; call `settle()` to fire `seeked` for all of them. */
  readonly settle: () => void;

  /** Make the next behaviors fail. */
  readonly fail: {
    context: boolean;
    draw: boolean;
    encode: "null" | "throw" | false;
    seek: boolean;
  };

  /** Initial `readyState` for new videos. */
  readyState: number;

  /** Restore the originals. */
  readonly restore: () => void;
}

/** Install the fakes; `autoSeek` fires `seeked` in a microtask instead of on `settle()`. */
export function installFakeCapture(autoSeek = true): FakeCaptureEnvironment {
  const video = HTMLMediaElement.prototype;
  const canvas = HTMLCanvasElement.prototype;
  const saved = new Map<object, Map<PropertyKey, PropertyDescriptor | undefined>>();
  const times = new WeakMap<HTMLMediaElement, number>();
  const pending: HTMLMediaElement[] = [];
  let urlCounter = 0;
  const originalCreate = Object.getOwnPropertyDescriptor(URL, "createObjectURL");
  const originalRevoke = Object.getOwnPropertyDescriptor(URL, "revokeObjectURL");

  function define(target: object, name: PropertyKey, descriptor: PropertyDescriptor): void {
    let entries = saved.get(target);
    if (entries === undefined) {
      entries = new Map();
      saved.set(target, entries);
    }
    if (!entries.has(name)) entries.set(name, Object.getOwnPropertyDescriptor(target, name));
    Object.defineProperty(target, name, { configurable: true, ...descriptor });
  }

  const environment: FakeCaptureEnvironment = {
    created: [],
    revoked: [],
    drawn: [],
    fail: { context: false, draw: false, encode: false, seek: false },
    readyState: 4,
    settle() {
      for (const element of pending.splice(0)) {
        element.dispatchEvent(new Event(environment.fail.seek ? "error" : "seeked"));
      }
    },
    restore() {
      for (const [target, entries] of saved) {
        for (const [name, descriptor] of entries) {
          if (descriptor) Object.defineProperty(target, name, descriptor);
          else Reflect.deleteProperty(target, name);
        }
      }
      for (const [name, descriptor] of [
        ["createObjectURL", originalCreate],
        ["revokeObjectURL", originalRevoke],
      ] as const) {
        if (descriptor) Object.defineProperty(URL, name, descriptor);
        else Reflect.deleteProperty(URL, name);
      }
    },
  };

  define(video, "readyState", {
    get: () => environment.readyState,
  });
  define(video, "duration", { get: () => 120 });
  define(video, "currentTime", {
    get(this: HTMLMediaElement) {
      return times.get(this) ?? 0;
    },
    set(this: HTMLMediaElement, value: number) {
      times.set(this, value);
      pending.push(this);
      if (autoSeek) void Promise.resolve().then(() => environment.settle());
    },
  });
  define(HTMLVideoElement.prototype, "videoWidth", { get: () => 640 });
  define(HTMLVideoElement.prototype, "videoHeight", { get: () => 360 });
  define(canvas, "getContext", {
    value(this: HTMLCanvasElement) {
      if (environment.fail.context) return null;
      return {
        drawImage: (source: HTMLMediaElement) => {
          if (environment.fail.draw) throw new DOMException("tainted", "SecurityError");
          environment.drawn.push(times.get(source) ?? 0);
        },
      };
    },
  });
  define(canvas, "toBlob", {
    value(this: HTMLCanvasElement, callback: BlobCallback, type?: string) {
      if (environment.fail.encode === "throw") throw new DOMException("tainted", "SecurityError");
      const size = `${this.width}x${this.height}`;
      callback(environment.fail.encode === "null" ? null : new Blob([size], { type: type ?? "" }));
    },
  });
  define(canvas, "toDataURL", {
    value(this: HTMLCanvasElement, type?: string) {
      return `data:${type ?? "image/png"};base64,${btoa(`${this.width}x${this.height}`)}`;
    },
  });
  URL.createObjectURL = () => {
    const url = `blob:https://app.test/frame-${++urlCounter}`;
    environment.created.push(url);
    return url;
  };
  URL.revokeObjectURL = (url: string) => {
    environment.revoked.push(url);
  };
  return environment;
}
