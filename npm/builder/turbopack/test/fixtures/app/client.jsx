"use client";
import { useEffect, useRef } from "react";
import { createSSRApp } from "vue";
import App from "../components/组件 space/FixtureCounter.vue";

export default function Client({ html }) {
  const root = useRef(null);
  // Once Vue hydrates this island, its renderer owns these descendants.
  const initialHtml = useRef(html);
  useEffect(() => {
    window.fixtureToken = crypto.randomUUID();
    const app = createSSRApp(App, { initial: 2 });
    app.mount(root.current);
    return () => app.unmount();
  }, []);
  return (
    <main id="vue-root" ref={root} dangerouslySetInnerHTML={{ __html: initialHtml.current }} />
  );
}
