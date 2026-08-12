import type { PressPointerType } from "./press-types.ts";

interface PendingActivation {
  readonly pointerType: PressPointerType;
  readonly target: Element;
  readonly timer: ReturnType<typeof setTimeout>;
}

export interface SyntheticActivation {
  readonly event: MouseEvent;
  readonly pointerType: PressPointerType;
  readonly target: Element;
  canceled: boolean;
}

/** Own short-lived tokens that associate release, cancellation, and click. */
export class PressActivationMemory {
  #pending: PendingActivation | null = null;
  #suppressedTarget: Element | null = null;
  #suppressedTimer: ReturnType<typeof setTimeout> | null = null;

  remember(target: Element, pointerType: PressPointerType): void {
    this.#clearPending();
    const timer = setTimeout(() => {
      if (this.#pending?.timer === timer) this.#pending = null;
    }, 1_000);
    this.#pending = { target, pointerType, timer };
  }

  take(target: Element): PressPointerType | null {
    if (this.#pending?.target !== target) return null;
    const pointerType = this.#pending.pointerType;
    this.#clearPending();
    return pointerType;
  }

  suppress(target: Element): void {
    this.#suppressedTarget = target;
    if (this.#suppressedTimer) clearTimeout(this.#suppressedTimer);
    this.#suppressedTimer = setTimeout(() => {
      this.#suppressedTarget = null;
      this.#suppressedTimer = null;
    }, 1_000);
  }

  consumeSuppressed(target: Element): boolean {
    if (this.#suppressedTarget !== target) return false;
    this.#suppressedTarget = null;
    if (this.#suppressedTimer) clearTimeout(this.#suppressedTimer);
    this.#suppressedTimer = null;
    return true;
  }

  begin(target: Element): void {
    this.#clearPending();
    this.consumeSuppressed(target);
  }

  dispose(): void {
    this.#clearPending();
    if (this.#suppressedTimer) clearTimeout(this.#suppressedTimer);
    this.#suppressedTimer = null;
    this.#suppressedTarget = null;
  }

  #clearPending(): void {
    if (!this.#pending) return;
    clearTimeout(this.#pending.timer);
    this.#pending = null;
  }
}
