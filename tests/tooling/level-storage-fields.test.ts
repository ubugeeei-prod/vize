import assert from "node:assert/strict";
import { test } from "node:test";

import { scanStorage } from "./davinci-storage-scan.ts";

test("test-only named fields preserve every neighboring production storage site", () => {
  const fields = [
    "probe: alloc::vec::Vec<u8>,",
    "pub probe: Result<alloc::vec::Vec<u8>, alloc::vec::Vec<u16>>,",
    "pub(crate) probe: fn(u8, u16) -> Result<alloc::vec::Vec<u8>, u8>,",
    "pub(in crate::inner) r#type: Result<fn() -> alloc::vec::Vec<u8>, u8>,",
    "probe: [alloc::vec::Vec<u8>; { let (a, b) = (1, 2); a + b }],",
    "probe: [u8; (1 < 2) as usize],",
    "probe: [u8; { (1 < 2) as usize }],",
    "probe: Option<fn(u8, u16) -> alloc::vec::Vec<u8>>,",
  ];
  for (const field of fields) {
    const result = scanStorage(`
      struct Owner {
        before: alloc::vec::Vec<u8>,
        #[cfg(test)] ${field}
        after: alloc::vec::Vec<u16>,
      }
      impl Owner { fn production(&self) -> Option<alloc::vec::Vec<u32>> { None } }
    `);
    assert.deepEqual(result.issues, []);
    assert.deepEqual(result.storage.allocVec, { directPaths: 3, boundUses: 0 }, field);
  }
});

test("a last test-only field leaves the outer close and later production items visible", () => {
  const result = scanStorage(`
    use alloc::vec::Vec;
    struct Owner {
      before: Vec<u8>,
      #[cfg(test)] probe: Result<Vec<u16>, Vec<u32>>
    }
    impl Owner { fn production(&self) -> Option<Vec<u64>> { None } }
  `);
  assert.deepEqual(result.issues, []);
  assert.deepEqual(result.storage.allocVec, { directPaths: 1, boundUses: 2 });
});

test("named initializer and consuming pattern fields do not hide the remaining original owner", () => {
  const result = scanStorage(`
    fn consume(owner: Owner) {
      let Owner { #[cfg(test)] probe: test_probe, original } = owner;
      let moved = Owner { #[cfg(test)] probe: test_probe, original };
      let _: Option<alloc::vec::Vec<u8>> = None;
    }
  `);
  assert.deepEqual(result.issues, []);
  assert.deepEqual(result.storage.allocVec, { directPaths: 1, boundUses: 0 });
});

test("shorthand pattern and initializer members leave their function and later items visible", () => {
  const result = scanStorage(`
    fn consume(owner: Owner) {
      let Owner { #[cfg(test)] probe, original } = owner;
      let moved = Owner { original, #[cfg(test)] probe };
      let _: Option<alloc::vec::Vec<u8>> = None;
    }
    type Production = alloc::vec::Vec<u16>;
  `);
  assert.deepEqual(result.issues, []);
  assert.deepEqual(result.storage.allocVec, { directPaths: 2, boundUses: 0 });
});

test("cfg function where-clause commas and other complete items keep their old boundary", () => {
  const items = [
    "fn helper<A, B>() where A: Into<B>, B: Copy { let _: Option<alloc::vec::Vec<u8>> = None; }",
    "mod tests { type Evidence = alloc::vec::Vec<u8>; }",
    "type Evidence = (alloc::vec::Vec<u8>, alloc::vec::Vec<u16>);",
    "const EVIDENCE: usize = { let _: Option<alloc::vec::Vec<u8>> = None; 1 };",
    "extern crate alloc;",
    "'label: loop { let _: Option<alloc::vec::Vec<u8>> = None; break 'label; }",
  ];
  for (const item of items) {
    const result = scanStorage(`
      #[cfg(test)] ${item}
      type Production = alloc::vec::Vec<u32>;
    `);
    assert.deepEqual(result.issues, []);
    assert.deepEqual(result.storage.allocVec, { directPaths: 1, boundUses: 0 }, item);
  }
});
