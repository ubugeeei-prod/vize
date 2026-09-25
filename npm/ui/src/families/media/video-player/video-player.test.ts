import assert from "node:assert/strict";

import { afterEach, beforeEach, test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { MediaPlayerRootExpose } from "../media-player/media-player.ts";
import { installFakeMedia } from "../media-player/media-player-test-utils.ts";
import type { FakeMedia } from "../media-player/media-player-test-utils.ts";
import { VideoPlayer, VideoPlayerPlayButton, VideoPlayerRoot } from "./video-player.ts";
import VideoPlayerFullscreenButton from "./video-player-fullscreen-button.vue";
import VideoPlayerPictureInPictureButton from "./video-player-picture-in-picture-button.vue";
import VideoPlayerVideo from "./video-player-video.vue";
import { mountInteraction } from "../../../testing/mount.ts";

let fake: FakeMedia;
const restorers: (() => void)[] = [];

beforeEach(() => {
  fake = installFakeMedia();
});

afterEach(() => {
  fake.restore();
  while (restorers.length > 0) restorers.pop()?.();
});

/** Define a property for one test and restore the original afterwards. */
function stub(target: object, name: string, descriptor: PropertyDescriptor): void {
  const original = Object.getOwnPropertyDescriptor(target, name);
  Object.defineProperty(target, name, { configurable: true, ...descriptor });
  restorers.push(() => {
    if (original) Object.defineProperty(target, name, original);
    else Reflect.deleteProperty(target, name);
  });
}

async function settle(): Promise<void> {
  await nextTick();
  await Promise.resolve();
  await nextTick();
}

function mountVideo(
  videoProps: Record<string, unknown> = {},
  rootProps: Record<string, unknown> = {},
  buttonProps: Record<string, unknown> = {},
) {
  return mountInteraction(VideoPlayerRoot, {
    props: { id: "trailer", ...rootProps },
    slots: {
      default: () => [
        h(VideoPlayerVideo, { src: "https://cdn.test/trailer.mp4", ...videoProps }, () =>
          h("track", { kind: "captions", src: "/en.vtt", srclang: "en", label: "English" }),
        ),
        h(VideoPlayerPlayButton),
        h(VideoPlayerFullscreenButton, buttonProps),
        h(VideoPlayerPictureInPictureButton, buttonProps),
      ],
    },
  });
}

function video(root: HTMLElement): HTMLVideoElement {
  const element = root.querySelector("video");
  assert.ok(element);
  return element;
}

test("renders a native video with validated sources and inline-playback attributes", async () => {
  const handle = mountVideo({
    poster: "/poster.jpg",
    width: 640,
    height: 360,
    crossOrigin: "anonymous",
    loop: true,
  });
  const root = handle.root();
  await settle();
  const element = video(root);

  assert.equal(root.getAttribute("data-media-kind"), "video");
  assert.equal(element.id, "trailer-media");
  assert.equal(element.getAttribute("data-vize-ui"), "video-player-video");
  assert.equal(element.getAttribute("src"), "https://cdn.test/trailer.mp4");
  assert.equal(element.getAttribute("poster"), "/poster.jpg");
  assert.equal(element.getAttribute("preload"), "metadata");
  assert.equal(element.hasAttribute("playsinline"), true);
  assert.equal(element.hasAttribute("loop"), true);
  assert.equal(element.hasAttribute("controls"), false);
  assert.equal(element.getAttribute("crossorigin"), "anonymous");
  assert.equal(element.getAttribute("width"), "640");
  assert.equal(element.querySelector("track")?.getAttribute("kind"), "captions");
  handle.unmount();

  const unsafe = mountVideo({
    src: "javascript:alert(1)",
    poster: "data:text/html;base64,PGgxPg==",
  });
  await settle();
  assert.equal(video(unsafe.root()).hasAttribute("src"), false);
  assert.equal(video(unsafe.root()).hasAttribute("poster"), false);
  assert.equal(video(unsafe.root()).getAttribute("data-invalid-src"), "true");
  unsafe.unmount();

  const insecure = mountVideo({ src: "http://localhost/a.mp4", allowInsecure: true });
  await settle();
  assert.equal(video(insecure.root()).getAttribute("src"), "http://localhost/a.mp4");
  insecure.unmount();
});

test("defaultMuted mutes the element for muted autoplay", async () => {
  const handle = mountVideo({ autoplay: true }, { defaultMuted: true });
  await settle();
  const element = video(handle.root());
  assert.equal(element.muted, true);
  assert.equal(element.hasAttribute("autoplay"), true);
  assert.equal(handle.root().getAttribute("data-muted"), "true");
  handle.unmount();
});

test("fullscreen toggles the root through the standard API and reports state", async () => {
  let fullscreenElement: Element | null = null;
  const requests: Element[] = [];
  stub(document, "fullscreenEnabled", { get: () => true });
  stub(document, "fullscreenElement", { get: () => fullscreenElement });
  stub(Element.prototype, "requestFullscreen", {
    value(this: Element) {
      requests.push(this);
      fullscreenElement = requests.at(-1) ?? null;
      document.dispatchEvent(new Event("fullscreenchange"));
      return Promise.resolve();
    },
  });
  stub(document, "exitFullscreen", {
    value() {
      fullscreenElement = null;
      document.dispatchEvent(new Event("fullscreenchange"));
      return Promise.resolve();
    },
  });

  const handle = mountVideo();
  const root = handle.root();
  await settle();
  const button = handle.getByRole("button", { name: "Enter fullscreen" }) as HTMLButtonElement;
  assert.equal(button.disabled, false);
  assert.equal(button.getAttribute("data-supported"), "true");
  await handle.click(button);
  await settle();
  assert.ok(requests[0] === root, "the root container, not the video, goes fullscreen");
  assert.equal(root.getAttribute("data-fullscreen"), "true");
  assert.equal(button.getAttribute("aria-label"), "Exit fullscreen");
  assert.equal(button.getAttribute("data-active"), "true");

  const shortcut = new KeyboardEvent("keydown", { key: "f", bubbles: true, cancelable: true });
  video(root).dispatchEvent(shortcut);
  await settle();
  assert.equal(shortcut.defaultPrevented, true);
  assert.equal(root.hasAttribute("data-fullscreen"), false);
  handle.unmount();
});

test("fullscreen falls back to WebKit prefixes and native video fullscreen", async () => {
  stub(document, "fullscreenEnabled", { get: () => false });
  const calls: string[] = [];
  stub(HTMLVideoElement.prototype, "webkitEnterFullscreen", {
    value(this: HTMLVideoElement) {
      calls.push("video");
      this.dispatchEvent(new Event("webkitbeginfullscreen"));
    },
  });
  const handle = mountVideo();
  await settle();
  const exposed = handle.exposes<MediaPlayerRootExpose>();
  assert.equal(await exposed.toggleFullscreen(), true);
  await settle();
  assert.deepEqual(calls, ["video"]);
  assert.equal(exposed.fullscreen, true);
  video(handle.root()).dispatchEvent(new Event("webkitendfullscreen"));
  await settle();
  assert.equal(exposed.fullscreen, false);
  handle.unmount();

  stub(HTMLElement.prototype, "webkitRequestFullscreen", {
    value() {
      calls.push("root");
    },
  });
  const prefixed = mountVideo();
  await settle();
  assert.equal(await prefixed.exposes<MediaPlayerRootExpose>().toggleFullscreen(), true);
  assert.deepEqual(calls, ["video", "root"]);
  prefixed.unmount();
});

test("picture-in-picture is detected after mount and toggles the video", async () => {
  let floating: Element | null = null;
  stub(document, "pictureInPictureEnabled", { get: () => true });
  stub(document, "pictureInPictureElement", { get: () => floating });
  stub(HTMLVideoElement.prototype, "requestPictureInPicture", {
    value(this: HTMLVideoElement) {
      this.dispatchEvent(new Event("enterpictureinpicture"));
      floating = this.ownerDocument.querySelector("video");
      return Promise.resolve({});
    },
  });
  stub(document, "exitPictureInPicture", {
    value() {
      const element = floating;
      floating = null;
      element?.dispatchEvent(new Event("leavepictureinpicture"));
      return Promise.resolve();
    },
  });

  const handle = mountVideo();
  const root = handle.root();
  await settle();
  const button = handle.getByRole("button", { name: "Enter picture-in-picture" });
  await handle.click(button);
  await settle();
  assert.equal(root.getAttribute("data-picture-in-picture"), "true");
  assert.equal(button.getAttribute("aria-label"), "Exit picture-in-picture");
  await handle.click(button);
  await settle();
  assert.equal(root.hasAttribute("data-picture-in-picture"), false);
  handle.unmount();

  const disabled = mountVideo({ disablePictureInPicture: true });
  await settle();
  const disabledButton = disabled.getByRole("button", {
    name: "Enter picture-in-picture",
  }) as HTMLButtonElement;
  assert.equal(disabledButton.disabled, true, "disablePictureInPicture turns support off");
  disabled.unmount();
});

test("unsupported platform controls are disabled or hidden", async () => {
  const disabled = mountVideo();
  await settle();
  for (const name of ["Enter fullscreen", "Enter picture-in-picture"]) {
    const button = disabled.getByRole("button", { name }) as HTMLButtonElement;
    assert.equal(button.disabled, true);
    assert.equal(button.hidden, false);
    assert.equal(button.getAttribute("data-supported"), "false");
  }
  disabled.unmount();

  const hidden = mountVideo({}, {}, { unsupported: "hide" });
  await settle();
  const buttons = [
    ...hidden
      .root()
      .querySelectorAll<HTMLButtonElement>(
        '[data-vize-ui="video-player-fullscreen-button"], [data-vize-ui="video-player-picture-in-picture-button"]',
      ),
  ];
  assert.equal(buttons.length, 2);
  assert.ok(buttons.every((button) => button.hidden));
  hidden.unmount();
});

test("platform buttons honor preventDefault and aria-label overrides", async () => {
  stub(document, "fullscreenEnabled", { get: () => true });
  let requested = 0;
  stub(Element.prototype, "requestFullscreen", {
    value() {
      requested += 1;
      return Promise.resolve();
    },
  });
  const handle = mountVideo(
    {},
    {},
    { ariaLabel: "Theater", onClick: (event: MouseEvent) => event.preventDefault() },
  );
  await settle();
  const buttons = handle.root().querySelectorAll('[aria-label="Theater"]');
  assert.equal(buttons.length, 2);
  for (const button of buttons) {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  }
  await settle();
  assert.equal(requested, 0);
  handle.unmount();
});

test("VideoPlayer aliases the shared root and requires a provider for its parts", () => {
  assert.equal(VideoPlayer, VideoPlayerRoot);
  for (const part of [
    VideoPlayerVideo,
    VideoPlayerFullscreenButton,
    VideoPlayerPictureInPictureButton,
  ]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: MediaPlayer requires a matching provider/,
    );
  }
});
