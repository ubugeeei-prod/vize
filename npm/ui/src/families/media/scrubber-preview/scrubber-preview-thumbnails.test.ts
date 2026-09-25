import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  createFrameCache,
  findThumbnailCue,
  formatScrubberTime,
  parseThumbnailVtt,
  quantizeTime,
  ratioFromPointer,
  spriteFrame,
} from "./scrubber-preview-thumbnails.ts";

const VTT = `WEBVTT

1
00:00.000 --> 00:05.000
sprite.jpg#xywh=0,0,160,90

2
00:00:05.000 --> 00:00:10.000
sprite.jpg#xywh=pixel:160,0,160,90

broken
00:10.000 --> 00:09.000
bad.jpg

00:15.000 --> 00:20.000
https://cdn.test/still.jpg

00:20.000 --> 00:25.000

`;

test("parses WebVTT thumbnail cues with sprite regions and resolves relative URLs", () => {
  const cues = parseThumbnailVtt(VTT, "https://media.test/thumbs/track.vtt");
  assert.equal(cues.length, 3, "cues with inverted times or no payload are skipped");
  assert.deepEqual(cues[0], {
    start: 0,
    end: 5,
    src: "https://media.test/thumbs/sprite.jpg",
    region: { x: 0, y: 0, width: 160, height: 90 },
  });
  assert.deepEqual(cues[1]?.region, { x: 160, y: 0, width: 160, height: 90 });
  assert.equal(cues[1]?.start, 5);
  assert.deepEqual(cues[2], {
    start: 15,
    end: 20,
    src: "https://cdn.test/still.jpg",
    region: null,
  });
  assert.ok(Object.isFrozen(cues));
  assert.equal(parseThumbnailVtt(VTT)[0]?.src, "sprite.jpg", "no base keeps relative URLs");
  assert.equal(
    parseThumbnailVtt("WEBVTT\r\n\r\n01:00:00.500 --> 01:00:01.000\r\na.png")[0]?.start,
    3600.5,
  );
  assert.deepEqual(parseThumbnailVtt(""), []);
});

test("finds the cue covering a time by binary search", () => {
  const cues = parseThumbnailVtt(VTT);
  assert.equal(findThumbnailCue(cues, 0)?.start, 0);
  assert.equal(findThumbnailCue(cues, 4.999)?.start, 0);
  assert.equal(findThumbnailCue(cues, 5)?.start, 5);
  assert.equal(findThumbnailCue(cues, 12), undefined, "gaps have no thumbnail");
  assert.equal(findThumbnailCue(cues, 19)?.start, 15);
  assert.equal(findThumbnailCue(cues, 20), undefined, "end is exclusive");
  assert.equal(findThumbnailCue([], 1), undefined);
});

test("maps times to sprite frames across rows and sheets", () => {
  const sprite = { src: "sheet.jpg", columns: 3, rows: 2, interval: 10, width: 100, height: 50 };
  assert.deepEqual(spriteFrame(sprite, 0), {
    src: "sheet.jpg",
    region: { x: 0, y: 0, width: 100, height: 50 },
  });
  assert.deepEqual(spriteFrame(sprite, 45).region, { x: 100, y: 50, width: 100, height: 50 });
  const paged = { ...sprite, src: (sheet: number) => `sheet-${sheet}.jpg`, frameCount: 8 };
  assert.equal(spriteFrame(paged, 65).src, "sheet-1.jpg");
  assert.deepEqual(spriteFrame(paged, 65).region, { x: 0, y: 0, width: 100, height: 50 });
  assert.deepEqual(spriteFrame(paged, 999).region, { x: 100, y: 0, width: 100, height: 50 });
  assert.deepEqual(spriteFrame(sprite, -5).region?.x, 0);
  assert.throws(
    () => spriteFrame({ ...sprite, interval: 0 }, 1),
    /VIZE_UI_SCRUBBER_PREVIEW_SPRITE/,
  );
});

test("maps pointers to ratios, quantizes times, and formats durations", () => {
  const rect = { left: 100, width: 200 };
  assert.equal(ratioFromPointer(150, rect), 0.25);
  assert.equal(ratioFromPointer(150, rect, "rtl"), 0.75);
  assert.equal(ratioFromPointer(0, rect), 0);
  assert.equal(ratioFromPointer(900, rect), 1);
  assert.equal(ratioFromPointer(150, { left: 0, width: 0 }), 0);

  assert.equal(quantizeTime(7.9, 2), 6);
  assert.equal(quantizeTime(-3, 2), 0);
  assert.equal(quantizeTime(1.5, 0), 1.5);

  assert.equal(formatScrubberTime(0), "0:00");
  assert.equal(formatScrubberTime(65.9), "1:05");
  assert.equal(formatScrubberTime(3723), "1:02:03");
  assert.equal(formatScrubberTime(59, true), "0:00:59");
  assert.equal(formatScrubberTime(Number.NaN), "0:00");
  assert.equal(formatScrubberTime(-4), "0:00");
});

test("the frame cache evicts least-recently-used URLs", () => {
  const evicted: string[] = [];
  const cache = createFrameCache(2, (url) => evicted.push(url));
  cache.set(1, "a");
  cache.set(2, "b");
  assert.equal(cache.get(1), "a", "reading refreshes recency");
  cache.set(3, "c");
  assert.deepEqual(evicted, ["b"]);
  assert.equal(cache.get(2), undefined);
  cache.set(3, "c2");
  assert.deepEqual(evicted, ["b", "c"], "replacing an entry releases the old URL");
  assert.equal(cache.size(), 2);
  cache.clear();
  assert.deepEqual(evicted, ["b", "c", "a", "c2"]);
  assert.equal(cache.size(), 0);
  assert.equal(createFrameCache(0, () => undefined).size(), 0);
});
