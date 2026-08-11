import { shallowReadonly, shallowRef } from "vue";

import { PressActivationMemory } from "./press-activation-memory.ts";
import type { SyntheticActivation } from "./press-activation-memory.ts";
import {
  createPressEvent,
  disableTextSelection,
  isEventInside,
  readBooleanOption,
  validatePressOptions,
} from "./press-event.ts";
import { captureError, notifyAll, surfaceErrors } from "./press-notify.ts";
import type {
  PressController,
  PressEvent,
  PressOptions,
  PressPointerType,
  PressProps,
} from "./press-types.ts";

const disposedDiagnostic = "VIZE_UI_PRESS_DISPOSED";

export type PressSource = "keyboard" | "mouse" | "pointer" | "touch";

export interface ActivePress {
  readonly document: Document;
  readonly id: number | null;
  readonly key: string | null;
  readonly nativeKeyboard: boolean;
  readonly pointerType: PressPointerType;
  readonly releaseListeners: () => void;
  readonly restoreSelection: () => void;
  readonly source: PressSource;
  readonly target: Element;
  delivered: boolean;
  inside: boolean;
  lastEvent: Event;
}

/** Internal press state machine shared by all native-event adapters. */
export class PressLifecycle {
  readonly options: PressOptions;
  readonly installListeners: (document: Document, source: PressSource) => () => void;
  readonly #pressed = shallowRef(false);
  readonly #activation = new PressActivationMemory();
  #active: ActivePress | null = null;
  #disposed = false;
  #synthetic: SyntheticActivation | null = null;
  #transitionVersion = 0;

  constructor(
    options: PressOptions,
    installListeners: (document: Document, source: PressSource) => () => void,
  ) {
    validatePressOptions(options);
    this.options = options;
    this.installListeners = installListeners;
  }

  get active(): ActivePress | null {
    return this.#active;
  }

  get disposed(): boolean {
    return this.#disposed;
  }

  start(
    event: Event,
    target: Element,
    source: PressSource,
    pointerType: PressPointerType,
    id: number | null,
    key: string | null,
    nativeKeyboard: boolean,
  ): void {
    this.#activation.begin(target);
    let releaseListeners: () => void = () => undefined;
    let restoreSelection: () => void = () => undefined;
    try {
      releaseListeners = this.installListeners(target.ownerDocument, source);
      if (
        source !== "keyboard" &&
        !readBooleanOption(this.options.allowTextSelectionOnPress, "allowTextSelectionOnPress")
      ) {
        restoreSelection = disableTextSelection(target);
      }
    } catch (error) {
      releaseListeners();
      restoreSelection();
      throw error;
    }
    const current: ActivePress = {
      document: target.ownerDocument,
      id,
      key,
      nativeKeyboard,
      pointerType,
      releaseListeners,
      restoreSelection,
      source,
      target,
      delivered: false,
      inside: true,
      lastEvent: event,
    };
    this.#active = current;
    this.#transition(
      true,
      createPressEvent(
        "pressstart",
        target,
        pointerType,
        event,
        false,
        source === "touch" ? id : null,
      ),
    );
  }

  updatePointerBoundary(current: ActivePress, event: Event): void {
    current.lastEvent = event;
    if (readBooleanOption(this.options.isDisabled, "isDisabled")) {
      this.cancelActive(event);
      return;
    }
    const inside = isEventInside(
      event,
      current.target,
      current.source === "touch" ? current.id : null,
    );
    if (inside === current.inside) return;
    current.inside = inside;
    if (
      !inside &&
      readBooleanOption(this.options.shouldCancelOnPointerExit, "shouldCancelOnPointerExit")
    ) {
      this.cancelActive(event);
      return;
    }
    this.#transition(
      inside,
      createPressEvent(
        inside ? "pressstart" : "pressend",
        current.target,
        current.pointerType,
        event,
        !inside,
        current.source === "touch" ? current.id : null,
      ),
    );
  }

  finishPointer(current: ActivePress, event: Event): void {
    current.lastEvent = event;
    const inside =
      current.target.isConnected &&
      current.inside &&
      isEventInside(event, current.target, current.source === "touch" ? current.id : null);
    if (readBooleanOption(this.options.isDisabled, "isDisabled") || !inside) {
      this.cancelActive(event);
      return;
    }
    const errors: unknown[] = [];
    captureError(errors, () => this.#emitUp(current, event));
    if (this.#active === current) {
      this.#activation.remember(current.target, current.pointerType);
      captureError(errors, () => this.#endActive(current, event, false));
    }
    surfaceErrors(errors);
  }

  finishKeyboard(current: ActivePress, event: KeyboardEvent): void {
    current.lastEvent = event;
    const disabled =
      !current.target.isConnected || readBooleanOption(this.options.isDisabled, "isDisabled");
    const errors: unknown[] = [];
    let completed = false;
    if (!disabled) captureError(errors, () => this.#emitUp(current, event));
    if (this.#active === current) {
      completed = !disabled;
      captureError(errors, () => this.#endActive(current, event, disabled));
    }
    if (completed && !this.#disposed && !current.delivered) {
      if (current.nativeKeyboard) this.#activation.remember(current.target, "keyboard");
      else captureError(errors, () => this.#emitPress(current.target, "keyboard", event));
    }
    surfaceErrors(errors);
  }

  activateClick(target: Element, event: MouseEvent): void {
    if (readBooleanOption(this.options.isDisabled, "isDisabled")) {
      event.preventDefault();
      this.cancelActive(event);
      this.#activation.dispose();
      return;
    }
    if (this.#activation.consumeSuppressed(target)) {
      event.preventDefault();
      return;
    }
    const current = this.#active;
    if (current?.target === target && current.source === "keyboard") {
      const shouldDeliver = !current.delivered;
      current.delivered = true;
      if (shouldDeliver) this.#emitPress(target, "keyboard", event);
      return;
    }
    if (current?.target === target) this.finishPointer(current, event);
    const pendingPointerType = this.#activation.take(target);
    if (pendingPointerType) {
      this.#emitPress(target, pendingPointerType, event);
      return;
    }
    this.#syntheticClickCycle(target, event, event.detail === 0 ? "virtual" : "mouse");
  }

  cancelActive(originalEvent: Event | null, suppress = true): boolean {
    const current = this.#active;
    if (current) {
      if (suppress && current.source !== "keyboard") this.#activation.suppress(current.target);
      this.#endActive(current, originalEvent ?? current.lastEvent, true);
      return true;
    }
    const synthetic = this.#synthetic;
    if (!synthetic || synthetic.canceled) return false;
    synthetic.canceled = true;
    this.#transition(
      false,
      createPressEvent(
        "pressend",
        synthetic.target,
        synthetic.pointerType,
        originalEvent ?? synthetic.event,
        true,
      ),
    );
    return true;
  }

  toController(pressProps: Readonly<PressProps>): PressController {
    return Object.freeze({
      isPressed: shallowReadonly(this.#pressed),
      pressProps,
      cancel: () => {
        if (this.#disposed) {
          throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
        }
        return this.cancelActive(null);
      },
      dispose: () => this.dispose(),
    });
  }

  dispose(): void {
    if (this.#disposed) return;
    const current = this.#active;
    if (current) {
      current.releaseListeners();
      current.restoreSelection();
      this.#active = null;
      this.#pressed.value = false;
      this.#transitionVersion++;
    }
    if (this.#synthetic) this.#synthetic.canceled = true;
    this.#synthetic = null;
    if (this.#pressed.value) {
      this.#pressed.value = false;
      this.#transitionVersion++;
    }
    this.#activation.dispose();
    this.#disposed = true;
  }

  #transition(next: boolean, event: PressEvent): void {
    if (this.#pressed.value === next) return;
    this.#pressed.value = next;
    const version = ++this.#transitionVersion;
    const phase = next ? this.options.onPressStart : this.options.onPressEnd;
    notifyAll([
      () => phase?.(event),
      () => {
        if (this.#transitionVersion === version) this.options.onPressChange?.(next);
      },
    ]);
  }

  #endActive(current: ActivePress, originalEvent: Event | null, canceled: boolean): void {
    current.releaseListeners();
    current.restoreSelection();
    if (this.#active === current) this.#active = null;
    this.#transition(
      false,
      createPressEvent(
        "pressend",
        current.target,
        current.pointerType,
        originalEvent,
        canceled,
        current.source === "touch" ? current.id : null,
      ),
    );
  }

  #emitUp(current: ActivePress, event: Event): void {
    this.options.onPressUp?.(
      createPressEvent(
        "pressup",
        current.target,
        current.pointerType,
        event,
        false,
        current.source === "touch" ? current.id : null,
      ),
    );
  }

  #emitPress(target: Element, pointerType: PressPointerType, event: Event): void {
    this.options.onPress?.(createPressEvent("press", target, pointerType, event));
  }

  #syntheticClickCycle(target: Element, event: MouseEvent, pointerType: PressPointerType): void {
    const errors: unknown[] = [];
    const cycle: SyntheticActivation = { event, pointerType, target, canceled: false };
    this.#synthetic = cycle;
    captureError(errors, () =>
      this.#transition(true, createPressEvent("pressstart", target, pointerType, event)),
    );
    if (!cycle.canceled && !this.#disposed) {
      captureError(errors, () =>
        this.options.onPressUp?.(createPressEvent("pressup", target, pointerType, event)),
      );
    }
    if (!cycle.canceled && !this.#disposed) {
      captureError(errors, () =>
        this.#transition(false, createPressEvent("pressend", target, pointerType, event)),
      );
    }
    if (!cycle.canceled && !this.#disposed) {
      captureError(errors, () => this.#emitPress(target, pointerType, event));
    }
    if (this.#synthetic === cycle) this.#synthetic = null;
    surfaceErrors(errors);
  }
}
