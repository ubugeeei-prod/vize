import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  applyMentionEdit,
  containsMentionFilter,
  defaultMentionInsert,
  detectMention,
  isSameMention,
  normalizeMentionText,
} from "./mention-core.ts";
import type { MentionTrigger } from "./mention-core.ts";

const at: MentionTrigger = { char: "@" };
const hash: MentionTrigger = { char: "#", pattern: /^[\w-]*$/u };
const colon: MentionTrigger = { char: ":", minChars: 2 };
const spaced: MentionTrigger = { allowSpaces: true, char: "@" };

test("detects a trigger at the start and after whitespace or punctuation", () => {
  assert.deepEqual(detectMention("@ad", 3), {
    end: 3,
    query: "ad",
    start: 0,
    trigger: { char: "@" },
  });
  assert.equal(detectMention("hi @ad", 6)?.query, "ad");
  assert.equal(detectMention("(@ad", 4)?.start, 1);
  assert.equal(detectMention("hi @", 4)?.query, "");
});

test("never triggers mid-word, after a repeated trigger, or across whitespace and newlines", () => {
  assert.equal(detectMention("mail@example", 12), null);
  assert.equal(detectMention("@@ad", 4), null);
  assert.equal(detectMention("@ada lovelace", 13), null);
  assert.equal(detectMention("@ada\nx", 6), null);
  assert.equal(detectMention("@ada ", 5), null);
  assert.equal(detectMention("@ad", 5), null, "out-of-range caret");
});

test("uses the caret position, not the end of the text", () => {
  const text = "hi @ad and more";
  assert.equal(detectMention(text, 6)?.query, "ad");
  assert.equal(detectMention(text, 3), null);
});

test("supports several triggers with patterns, minimum length, and spaces", () => {
  const triggers = [at, hash, colon];
  assert.equal(detectMention("see #bug-12", 11, triggers)?.trigger, hash);
  assert.equal(detectMention("see #bug!", 9, triggers), null, "pattern rejects the query");
  assert.equal(detectMention("hi :s", 5, triggers), null, "below minChars");
  assert.equal(detectMention("hi :sm", 6, triggers)?.trigger, colon);
  assert.equal(detectMention("@ada lov", 8, [spaced])?.query, "ada lov");
  assert.equal(detectMention("@ ada", 5, [spaced]), null, "leading space never opens");
  assert.equal(detectMention("@ab", 3, [{ char: "@", maxChars: 1 }]), null);
});

test("multi-character triggers match as a unit", () => {
  const trigger: MentionTrigger = { char: "{{" };
  assert.equal(detectMention("x {{na", 6, [trigger])?.query, "na");
  assert.equal(detectMention("x {na", 5, [trigger]), null);
});

test("applies edits with the default insertion and collapses a doubled space", () => {
  const text = "hi @ad there";
  const match = detectMention(text, 6);
  if (match === null) assert.fail("match expected");
  const edit = applyMentionEdit(text, match, defaultMentionInsert("ada", at));
  assert.equal(edit.text, "hi @ada there");
  assert.equal(edit.caret, 8);
  assert.equal(edit.inserted, "@ada ");
  assert.deepEqual([edit.start, edit.end], [3, 6]);
});

test("compares matches and normalizes filter text", () => {
  const left = detectMention("@a", 2);
  assert.equal(isSameMention(left, detectMention("@a", 2)), true);
  assert.equal(isSameMention(left, null), false);
  assert.equal(isSameMention(null, null), true);
  assert.equal(normalizeMentionText("ÉCOLE"), "ecole");
  assert.equal(containsMentionFilter("Zoë", "zoe"), true);
  assert.equal(containsMentionFilter("Ada", ""), true);
  assert.equal(containsMentionFilter("Ada", "x"), false);
});
