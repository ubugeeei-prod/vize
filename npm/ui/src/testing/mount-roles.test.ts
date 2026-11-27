import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { h } from "vue";
import { mountInteraction } from "./mount.ts";

test("queries native article, table, row, cell and named form roles", () => {
  const handle = mountInteraction({
    render: () =>
      h("section", [
        h("article", { "aria-label": "Message" }, "Hello"),
        h("table", { "aria-label": "Patch" }, [h("tbody", [h("tr", [h("td", "Changed")])])]),
        h("form", { "aria-label": "Composer" }),
      ]),
  });
  try {
    assert.equal(handle.getByRole("article", { name: "Message" }).tagName, "ARTICLE");
    assert.equal(handle.getByRole("table", { name: "Patch" }).tagName, "TABLE");
    assert.equal(handle.getByRole("row").tagName, "TR");
    assert.equal(handle.getByRole("cell", { name: "Changed" }).tagName, "TD");
    assert.equal(handle.getByRole("form", { name: "Composer" }).tagName, "FORM");
  } finally {
    handle.unmount();
  }
});

test("explicit roles remain authoritative and grid cells retain their context", () => {
  const handle = mountInteraction({
    render: () =>
      h("section", [
        h("article", { role: "group", "aria-label": "Explicit" }),
        h("table", { role: "grid" }, [h("tbody", [h("tr", [h("td", "Grid value")])])]),
      ]),
  });
  try {
    assert.equal(handle.queryByRole("article"), null);
    assert.equal(handle.queryByRole("table"), null);
    assert.equal(handle.queryByRole("cell"), null);
    assert.equal(handle.getByRole("group", { name: "Explicit" }).tagName, "ARTICLE");
    assert.equal(handle.getByRole("gridcell", { name: "Grid value" }).tagName, "TD");
  } finally {
    handle.unmount();
  }
});

test("unnamed forms do not become landmarks and valid labelledby names do", () => {
  const handle = mountInteraction({
    render: () =>
      h("section", [
        h("form", ["Ordinary contents"]),
        h("form", { "aria-label": " " }),
        h("form", { "aria-labelledby": "missing-label" }),
        h("span", { id: "named-form-label" }, "Named form"),
        h("form", { "aria-labelledby": "named-form-label" }),
      ]),
  });
  try {
    assert.equal(handle.getByRole("form", { name: "Named form" }).tagName, "FORM");
    assert.equal(handle.getByRole("form").getAttribute("aria-labelledby"), "named-form-label");
  } finally {
    handle.unmount();
  }
});

test("table captions name tables while ARIA labels take precedence", () => {
  const handle = mountInteraction({
    render: () =>
      h("section", [
        h("table", [h("caption", "Patch"), h("tbody", [h("tr", [h("td", "Changed")])])]),
        h("table", { "aria-label": "Explicit" }, [h("caption", "Other")]),
        h("span", { id: "table-label" }, "Referenced"),
        h("table", { "aria-labelledby": "table-label", "aria-label": "Ignored" }, [
          h("caption", "Also ignored"),
        ]),
      ]),
  });
  try {
    for (const name of ["Patch", "Explicit", "Referenced"])
      assert.equal(handle.getByRole("table", { name }).tagName, "TABLE");
  } finally {
    handle.unmount();
  }
});

for (const role of ["grid", "treegrid", "grid table", " grid "]) {
  test(`cell classification parses the owner role ${JSON.stringify(role)}`, () => {
    const handle = mountInteraction({
      render: () => h("table", { role }, [h("tbody", [h("tr", [h("td", "Value")])])]),
    });
    try {
      assert.equal(handle.getByRole("gridcell", { name: "Value" }).tagName, "TD");
      assert.equal(handle.queryByRole("cell"), null);
    } finally {
      handle.unmount();
    }
  });
}

test("form titles supply fallback names and ARIA names remain authoritative", () => {
  const handle = mountInteraction({
    render: () =>
      h("section", [
        h("form", { title: "Composer" }, "Ordinary contents"),
        h("form", { title: "Ignored", "aria-label": "Explicit" }),
        h("span", { id: "form-title-label" }, "Referenced"),
        h("form", { title: "Also ignored", "aria-labelledby": "form-title-label" }),
      ]),
  });
  try {
    for (const name of ["Composer", "Explicit", "Referenced"])
      assert.equal(handle.getByRole("form", { name }).tagName, "FORM");
  } finally {
    handle.unmount();
  }
});
