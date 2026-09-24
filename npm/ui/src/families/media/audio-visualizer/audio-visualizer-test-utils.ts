/**
 * Test-only Web Audio fakes. happy-dom has no Web Audio implementation, so
 * these model the parts the analyser touches: contexts with a `state`, nodes
 * that record connections, and an analyser that fills buffers with a frame
 * counter so tests can observe each read.
 */

/** Recording stand-in for `AudioNode`. */
export class FakeAudioNode {
  readonly connections: unknown[] = [];

  connect(target: unknown): unknown {
    this.connections.push(target);
    return target;
  }

  disconnect(target?: unknown): void {
    if (target === undefined) {
      this.connections.length = 0;
      return;
    }
    const index = this.connections.indexOf(target);
    if (index < 0) throw new Error("InvalidAccessError: not connected");
    this.connections.splice(index, 1);
  }
}

/** Stand-in for `AnalyserNode` that writes a recognizable pattern per read. */
export class FakeAnalyserNode extends FakeAudioNode {
  fftSize = 2048;
  smoothingTimeConstant = 0.8;
  minDecibels = -100;
  maxDecibels = -30;
  reads = 0;
  byteReads = 0;
  floatReads = 0;
  amplitude = 64;

  get frequencyBinCount(): number {
    return this.fftSize / 2;
  }

  getByteFrequencyData(array: Uint8Array): void {
    this.reads += 1;
    this.byteReads += 1;
    array.fill(Math.min(255, this.reads * 10));
  }

  getByteTimeDomainData(array: Uint8Array): void {
    for (let index = 0; index < array.length; index++) {
      array[index] = index % 2 === 0 ? 128 + this.amplitude : 128 - this.amplitude;
    }
  }

  getFloatFrequencyData(array: Float32Array): void {
    this.reads += 1;
    this.floatReads += 1;
    array.fill(-65);
  }

  getFloatTimeDomainData(array: Float32Array): void {
    for (let index = 0; index < array.length; index++) array[index] = index % 2 === 0 ? 0.5 : -0.5;
  }
}

/** Stand-in for `AudioContext`. */
export class FakeAudioContext extends EventTarget {
  static instances: FakeAudioContext[] = [];
  state: AudioContextState = "suspended";
  readonly sampleRate = 48_000;
  readonly destination = new FakeAudioNode();
  readonly analysers: FakeAnalyserNode[] = [];
  elementSources = 0;
  streamSources = 0;
  failAnalyser = false;

  constructor() {
    super();
    FakeAudioContext.instances.push(this);
  }

  createAnalyser(): FakeAnalyserNode {
    if (this.failAnalyser) throw new Error("NotSupportedError");
    const analyser = new FakeAnalyserNode();
    this.analysers.push(analyser);
    return analyser;
  }

  createMediaElementSource(_element: HTMLMediaElement): FakeAudioNode {
    this.elementSources += 1;
    return new FakeAudioNode();
  }

  createMediaStreamSource(_stream: MediaStream): FakeAudioNode {
    this.streamSources += 1;
    return new FakeAudioNode();
  }

  resume(): Promise<void> {
    this.state = "running";
    this.dispatchEvent(new Event("statechange"));
    return Promise.resolve();
  }
}

/** Stand-in for `MediaStream`. */
export class FakeMediaStream {
  getTracks(): readonly unknown[] {
    return [];
  }
}

/** Handle returned by {@link installFakeAudio}. */
export interface FakeAudioEnvironment {
  /** Run queued animation frames at `time` milliseconds. */
  readonly frame: (time: number) => void;

  /** Number of queued animation frames. */
  readonly queued: () => number;

  /** Restore the original globals. */
  readonly restore: () => void;
}

/** Install Web Audio and `requestAnimationFrame` fakes on `globalThis`. */
export function installFakeAudio(
  options: { readonly audio?: boolean; readonly reducedMotion?: boolean } = {},
): FakeAudioEnvironment {
  const names = [
    "AudioContext",
    "AudioNode",
    "MediaStream",
    "requestAnimationFrame",
    "cancelAnimationFrame",
    "matchMedia",
  ] as const;
  const originals = new Map(names.map((name) => [name, Reflect.get(globalThis, name)]));
  let callbacks = new Map<number, FrameRequestCallback>();
  let nextId = 1;
  FakeAudioContext.instances = [];

  if (options.audio !== false) {
    Reflect.set(globalThis, "AudioContext", FakeAudioContext);
    Reflect.set(globalThis, "AudioNode", FakeAudioNode);
    Reflect.set(globalThis, "MediaStream", FakeMediaStream);
  } else {
    Reflect.deleteProperty(globalThis, "AudioContext");
  }
  Reflect.set(globalThis, "requestAnimationFrame", (callback: FrameRequestCallback) => {
    const id = nextId++;
    callbacks.set(id, callback);
    return id;
  });
  Reflect.set(globalThis, "cancelAnimationFrame", (id: number) => callbacks.delete(id));
  Reflect.set(globalThis, "matchMedia", (query: string) => ({
    matches: options.reducedMotion === true && query.includes("reduce"),
    media: query,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
  }));

  return {
    frame(time) {
      const due = callbacks;
      callbacks = new Map();
      for (const callback of due.values()) callback(time);
    },
    queued: () => callbacks.size,
    restore() {
      for (const [name, value] of originals) {
        if (value === undefined) Reflect.deleteProperty(globalThis, name);
        else Reflect.set(globalThis, name, value);
      }
    },
  };
}
