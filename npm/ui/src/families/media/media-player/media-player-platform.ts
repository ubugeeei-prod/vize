/**
 * Feature-detected fullscreen and picture-in-picture helpers. Prefixed WebKit
 * APIs are reached through `in` checks and `Reflect`, never through casts.
 * Every helper is client-only; callers run them from handlers or `onMounted`.
 */

function callMethod(target: object, name: string): unknown {
  if (!(name in target)) return undefined;
  const method: unknown = Reflect.get(target, name);
  if (typeof method !== "function") return undefined;
  const result: unknown = Reflect.apply(method, target, []);
  return result ?? true;
}

function readProperty(target: object, name: string): unknown {
  return name in target ? Reflect.get(target, name) : undefined;
}

async function settle(result: unknown): Promise<boolean> {
  if (result === undefined) return false;
  if (result instanceof Promise) {
    try {
      await result;
    } catch {
      return false;
    }
  }
  return true;
}

/** Element currently shown fullscreen, including the WebKit-prefixed API. */
export function currentFullscreenElement(targetDocument: Document): Element | null {
  if (targetDocument.fullscreenElement) return targetDocument.fullscreenElement;
  const prefixed = readProperty(targetDocument, "webkitFullscreenElement");
  return prefixed instanceof Element ? prefixed : null;
}

/** Whether `element` (or, on iOS, `media`) can enter fullscreen. */
export function canRequestFullscreen(element: Element, media: HTMLMediaElement | null): boolean {
  if (typeof element.requestFullscreen === "function" && element.ownerDocument.fullscreenEnabled) {
    return true;
  }
  if (typeof readProperty(element, "webkitRequestFullscreen") === "function") return true;
  return media !== null && typeof readProperty(media, "webkitEnterFullscreen") === "function";
}

/** Enter fullscreen on `element`, falling back to native video fullscreen on iOS. */
export function requestElementFullscreen(
  element: Element,
  media: HTMLMediaElement | null,
): Promise<boolean> {
  if (typeof element.requestFullscreen === "function" && element.ownerDocument.fullscreenEnabled) {
    return settle(element.requestFullscreen());
  }
  const prefixed = callMethod(element, "webkitRequestFullscreen");
  if (prefixed !== undefined) return settle(prefixed);
  return settle(media === null ? undefined : callMethod(media, "webkitEnterFullscreen"));
}

/** Leave fullscreen, including the WebKit-prefixed API. */
export function exitDocumentFullscreen(targetDocument: Document): Promise<boolean> {
  if (typeof targetDocument.exitFullscreen === "function" && targetDocument.fullscreenElement) {
    return settle(targetDocument.exitFullscreen());
  }
  return settle(callMethod(targetDocument, "webkitExitFullscreen"));
}

/** Whether `media` can enter picture-in-picture. */
export function canRequestPictureInPicture(media: HTMLMediaElement | null): boolean {
  if (!(media instanceof HTMLVideoElement)) return false;
  const targetDocument = media.ownerDocument;
  return (
    readProperty(targetDocument, "pictureInPictureEnabled") === true &&
    readProperty(media, "disablePictureInPicture") !== true &&
    !media.hasAttribute("disablepictureinpicture") &&
    typeof readProperty(media, "requestPictureInPicture") === "function"
  );
}

/** Element currently floating in picture-in-picture. */
export function currentPictureInPictureElement(targetDocument: Document): Element | null {
  const element = readProperty(targetDocument, "pictureInPictureElement");
  return element instanceof Element ? element : null;
}

/** Enter or leave picture-in-picture for `media`. */
export function togglePictureInPictureFor(media: HTMLMediaElement): Promise<boolean> {
  const targetDocument = media.ownerDocument;
  if (currentPictureInPictureElement(targetDocument) === media) {
    return settle(callMethod(targetDocument, "exitPictureInPicture"));
  }
  if (!canRequestPictureInPicture(media)) return Promise.resolve(false);
  return settle(callMethod(media, "requestPictureInPicture"));
}
