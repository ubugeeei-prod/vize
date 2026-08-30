import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import BreadcrumbRoot from "./breadcrumb.vue";
import BreadcrumbItem from "./breadcrumb-item.vue";
import BreadcrumbLink from "./breadcrumb-link.vue";
import BreadcrumbList from "./breadcrumb-list.vue";
import BreadcrumbSeparator from "./breadcrumb-separator.vue";

const SsrProbe = defineComponent({
  name: "BreadcrumbSsrProbe",
  setup() {
    return () =>
      h(
        BreadcrumbRoot,
        { label: "Docs path" },
        {
          default: () =>
            h(BreadcrumbList, null, {
              default: () => [
                h(BreadcrumbItem, null, {
                  default: () => [
                    h(BreadcrumbLink, { href: "/" }, { default: () => "Home" }),
                    h(BreadcrumbSeparator, null, { default: () => "/" }),
                  ],
                }),
                h(
                  BreadcrumbItem,
                  { current: true },
                  {
                    default: () =>
                      h(BreadcrumbLink, { current: "page" }, { default: () => "Docs" }),
                  },
                ),
              ],
            }),
        },
      );
  },
});

test("renders byte-identical breadcrumb markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<nav/);
  assert.match(html, /aria-label="Docs path"/);
  assert.match(html, /part="root"/);
  assert.match(html, /data-vize-ui="breadcrumb"/);
  assert.match(html, /<ol/);
  assert.match(html, /data-vize-ui="breadcrumb-list"/);
  assert.match(html, /<li/);
  assert.match(html, /href="\//);
  assert.match(html, /aria-hidden="true"/);
  assert.match(html, /role="presentation"/);
  assert.match(html, /aria-current="page"/);
  assert.match(html, /data-current="true"/);
  assert.match(html, /Home/);
  assert.match(html, /Docs/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
  assert.doesNotMatch(html, /tabindex=/);
  assert.doesNotMatch(html, /aria-live=/);
});

test("renders consumer-owned route hosts without implicit list roles", async () => {
  const html = await renderToString(
    createSSRApp({
      name: "BreadcrumbCustomSsrProbe",
      setup() {
        return () =>
          h(
            BreadcrumbRoot,
            { as: "section", label: "Workspace path" },
            {
              default: () =>
                h(
                  BreadcrumbList,
                  { as: "ul" },
                  {
                    default: () =>
                      h(
                        BreadcrumbItem,
                        { as: "div", current: true },
                        {
                          default: () =>
                            h(
                              BreadcrumbLink,
                              { as: "span", current: "location" },
                              {
                                default: () => "Settings",
                              },
                            ),
                        },
                      ),
                  },
                ),
            },
          );
      },
    }),
  );

  assert.match(html, /^<section/);
  assert.match(html, /aria-label="Workspace path"/);
  assert.match(html, /<ul/);
  assert.match(html, /<div/);
  assert.match(html, /<span/);
  assert.match(html, /aria-current="location"/);
  assert.match(html, /data-current="true"/);
  assert.match(html, /Settings/);
  assert.doesNotMatch(html, /role="list"/);
  assert.doesNotMatch(html, /href=/);
  assert.doesNotMatch(html, /class=/);
  assert.doesNotMatch(html, /style=/);
});
