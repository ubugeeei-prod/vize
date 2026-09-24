/**
 * Test-only HTMLMediaElement model. happy-dom does not decode media, so this
 * installs deterministic playback state on `HTMLMediaElement.prototype`:
 * `play()`/`pause()` resolve and dispatch native events, setters dispatch the
 * matching change events, and helpers drive duration, buffering, the end of
 * playback, errors, and text tracks. Call `restore()` in a `finally` block.
 */

interface FakeTrack {
  kind: string;
  label: string;
  language: string;
  mode: TextTrackMode;
}

interface FakeState {
  paused: boolean;
  ended: boolean;
  seeking: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  muted: boolean;
  playbackRate: number;
  buffered: { start: number; end: number }[];
  error: { code: number } | null;
  tracks: FakeTextTrackList;
}

/** Minimal TextTrackList: an EventTarget with indexed tracks and `length`. */
export class FakeTextTrackList extends EventTarget {
  readonly items: FakeTrack[] = [];

  get length(): number {
    return this.items.length;
  }

  add(track: Omit<FakeTrack, "mode"> & { readonly mode?: TextTrackMode }): void {
    let mode: TextTrackMode = track.mode ?? "disabled";
    const notify = (): boolean => this.dispatchEvent(new Event("change"));
    const entry: FakeTrack = {
      kind: track.kind,
      label: track.label,
      language: track.language,
      get mode() {
        return mode;
      },
      set mode(next: TextTrackMode) {
        mode = next;
        notify();
      },
    };
    Object.defineProperty(this, String(this.items.length), {
      configurable: true,
      value: entry,
    });
    this.items.push(entry);
    this.dispatchEvent(new Event("addtrack"));
  }
}

/** Handle returned by {@link installFakeMedia}. */
export interface FakeMedia {
  /** `play()` calls, in order. */
  readonly playCalls: HTMLMediaElement[];
  /** Make the next `play()` calls reject like an autoplay policy. */
  rejectPlay: boolean;
  readonly state: (media: HTMLMediaElement) => FakeState;
  readonly setDuration: (media: HTMLMediaElement, duration: number) => void;
  readonly setBuffered: (media: HTMLMediaElement, end: number) => void;
  readonly advance: (media: HTMLMediaElement, time: number) => void;
  readonly end: (media: HTMLMediaElement) => void;
  readonly wait: (media: HTMLMediaElement) => void;
  readonly fail: (media: HTMLMediaElement, code: number) => void;
  readonly restore: () => void;
}

/** Install the fake media model on `HTMLMediaElement.prototype`. */
export function installFakeMedia(): FakeMedia {
  const prototype = HTMLMediaElement.prototype;
  const originals = new Map<string, PropertyDescriptor | undefined>();
  const states = new WeakMap<HTMLMediaElement, FakeState>();
  const playCalls: HTMLMediaElement[] = [];

  function state(media: HTMLMediaElement): FakeState {
    let current = states.get(media);
    if (current === undefined) {
      current = {
        paused: true,
        ended: false,
        seeking: false,
        currentTime: 0,
        duration: Number.NaN,
        volume: 1,
        muted: false,
        playbackRate: 1,
        buffered: [],
        error: null,
        tracks: new FakeTextTrackList(),
      };
      states.set(media, current);
    }
    return current;
  }

  function define(name: string, descriptor: PropertyDescriptor): void {
    if (!originals.has(name)) originals.set(name, Object.getOwnPropertyDescriptor(prototype, name));
    Object.defineProperty(prototype, name, { configurable: true, ...descriptor });
  }

  function emit(media: HTMLMediaElement, ...names: string[]): void {
    for (const name of names) media.dispatchEvent(new Event(name));
  }

  function accessor<Key extends keyof FakeState>(
    name: Key,
    onSet?: (media: HTMLMediaElement, value: FakeState[Key]) => void,
  ): void {
    const get = function (this: HTMLMediaElement): FakeState[Key] {
      return state(this)[name];
    };
    if (onSet === undefined) {
      define(name, { get });
      return;
    }
    define(name, {
      get,
      set(this: HTMLMediaElement, value: FakeState[Key]) {
        onSet(this, value);
      },
    });
  }

  accessor("paused");
  accessor("ended");
  accessor("seeking");
  accessor("duration");
  accessor("error");
  accessor("currentTime", (media, value) => {
    const current = state(media);
    current.currentTime = value;
    current.ended = false;
    emit(media, "seeking", "timeupdate", "seeked");
  });
  accessor("volume", (media, value) => {
    state(media).volume = value;
    emit(media, "volumechange");
  });
  accessor("muted", (media, value) => {
    state(media).muted = value;
    emit(media, "volumechange");
  });
  accessor("playbackRate", (media, value) => {
    state(media).playbackRate = value;
    emit(media, "ratechange");
  });
  define("buffered", {
    get(this: HTMLMediaElement) {
      const ranges = state(this).buffered;
      return {
        length: ranges.length,
        start: (index: number) => ranges[index]?.start ?? 0,
        end: (index: number) => ranges[index]?.end ?? 0,
      };
    },
  });
  define("textTracks", {
    get(this: HTMLMediaElement) {
      return state(this).tracks;
    },
  });

  const handle: FakeMedia = {
    playCalls,
    rejectPlay: false,
    state,
    setDuration(media, duration) {
      state(media).duration = duration;
      emit(media, "durationchange", "loadedmetadata");
    },
    setBuffered(media, end) {
      state(media).buffered = [{ start: 0, end }];
      emit(media, "progress");
    },
    advance(media, time) {
      state(media).currentTime = time;
      emit(media, "timeupdate");
    },
    end(media) {
      const current = state(media);
      current.paused = true;
      current.ended = true;
      current.currentTime = Number.isFinite(current.duration) ? current.duration : 0;
      emit(media, "pause", "ended");
    },
    wait(media) {
      emit(media, "waiting");
    },
    fail(media, code) {
      state(media).error = { code };
      emit(media, "error");
    },
    restore() {
      for (const [name, descriptor] of originals) {
        if (descriptor) Object.defineProperty(prototype, name, descriptor);
        else Reflect.deleteProperty(prototype, name);
      }
    },
  };

  define("play", {
    value(this: HTMLMediaElement): Promise<void> {
      playCalls.push(this);
      if (handle.rejectPlay) {
        return Promise.reject(new DOMException("blocked", "NotAllowedError"));
      }
      const current = state(this);
      if (current.ended) current.currentTime = 0;
      current.paused = false;
      current.ended = false;
      emit(this, "play", "playing");
      return Promise.resolve();
    },
  });
  define("pause", {
    value(this: HTMLMediaElement): void {
      const current = state(this);
      if (current.paused) return;
      current.paused = true;
      emit(this, "pause");
    },
  });

  return handle;
}
