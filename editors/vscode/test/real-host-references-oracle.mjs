import assert from "node:assert/strict";

// Mirrors `HOST_TEST_REFERENCES_COMMAND` in `src/host-test-core.ts`; the
// tooling tests assert both stay in sync.
export const HOST_TEST_REFERENCES_COMMAND = "vize.test.executeReferences";

const REQUIRED_REFERENCES = [
  { label: "script declaration", range: [3, 6, 3, 11] },
  { label: "prop binding usage", range: [9, 19, 9, 24] },
  { label: "interpolation usage", range: [10, 10, 10, 15] },
];

export function assertRealHostReferences(actual, { uri }) {
  assert.ok(Array.isArray(actual), "real host references must return locations");
  const locations = actual.map(describeLocation).sort(compareLocation);

  for (const location of locations) {
    assert.equal(
      location.uri,
      uri,
      `real host references must stay on the authored SFC, not a generated virtual document: ${JSON.stringify(locations)}`,
    );
  }

  for (const required of REQUIRED_REFERENCES) {
    assert.ok(
      locations.some((location) => rangesEqual(location.range, required.range)),
      `real host references must include the ${required.label}: ${JSON.stringify(locations)}`,
    );
  }

  assert.equal(
    locations.length,
    REQUIRED_REFERENCES.length,
    `real host references must not include extra generated or duplicate locations: ${JSON.stringify(locations)}`,
  );
  return locations;
}

function describeLocation(location) {
  return {
    range: [
      location.range.start.line,
      location.range.start.character,
      location.range.end.line,
      location.range.end.character,
    ],
    uri: location.uri,
  };
}

function rangesEqual(left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function compareLocation(left, right) {
  return left.range[0] - right.range[0] || left.range[1] - right.range[1];
}
