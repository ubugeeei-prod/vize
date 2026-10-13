import assert from "node:assert/strict";
export function styleState(value) {
  const { links: _links, hydrated: _hydrated, ...state } = value;
  return state;
}
export function verifyInitial(variant, value, css) {
  assert.equal(value.hydrated, "false", "no application JavaScript ran");
  if (variant === "baseline") {
    assert.equal(value.background, "rgba(0, 0, 0, 0)", "0.443.0 reproduces dev SSR flash");
    assert.equal(value.links.length, 0);
  } else {
    assert.equal(value.background, "rgb(18, 24, 30)");
    assert.equal(value.border, "rgb(40, 200, 90)");
    assert.equal(value.borderWidth, "3px");
    assert.equal(value.padding, "24px");
    assert.equal(value.width, 420);
    assert.equal(css.length > 0, true, "initial HTML must request its page stylesheet");
    for (const entry of css) {
      assert.equal(entry.status, 200, "real stylesheet GET must work");
      assert.match(entry.contentType, /text\/css/);
    }
  }
}
export function verifyHydration(initial, hydrated) {
  assert.equal(initial.links.length > 0 && hydrated.links.length > 0, true);
  assert.equal(initial.hydrated, "false");
  assert.equal(hydrated.hydrated, "true");
  assert.deepEqual(
    styleState(hydrated),
    styleState(initial),
    "hydration cannot alter authored CSS or card dimensions",
  );
}
export function verifyStock(published, stock) {
  assert.equal(published.links.length > 0 && stock.links.length > 0, true);
  assert.equal(published.hydrated, stock.hydrated);
  assert.deepEqual(
    styleState(published),
    styleState(stock),
    "published correction matches stock Vue's whole CSS and geometry",
  );
}
