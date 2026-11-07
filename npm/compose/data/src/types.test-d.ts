import {
  createDataClient,
  createFetchDataResource,
  defineDataResource,
  type InferData,
  type InferInput,
  type InferKey,
} from "./index.ts";

const article = defineDataResource({
  key: "article.byId",
  source: "../fixtures/ArticleView.vue",
  async loader({ input }: { input: { id: string } }) {
    return {
      id: input.id,
      title: "Intro",
    };
  },
});

type ArticleInput = InferInput<typeof article>;
type ArticleData = InferData<typeof article>;
type ArticleKey = InferKey<typeof article>;

const key: ArticleKey = "article.byId";
const input: ArticleInput = { id: "intro" };
const data: ArticleData = { id: "intro", title: "Intro" };
key satisfies "article.byId";
input satisfies { id: string };
data satisfies { id: string; title: string };

const client = createDataClient();
const loaded = await client.load(article, { id: "intro" });

if (loaded.status === "success") {
  loaded.data.title satisfies string;
}

await client.load(
  article,
  { id: "intro" },
  {
    deadlineMs: 500,
    retries: 2,
    retryDelayMs(context) {
      context.key satisfies "article.byId";
      context.input satisfies { id: string };
      context.attempt satisfies number;
      context.error satisfies unknown;
      return context.attempt * 10;
    },
  },
);

createDataClient({
  sleep(milliseconds, signal) {
    milliseconds satisfies number;
    signal satisfies AbortSignal;
    return Promise.resolve();
  },
});

client.optimistic(article, { id: "intro" }, (current) => ({
  id: current.id,
  title: current.title.toUpperCase(),
}));

const fetched = createFetchDataResource<"article.fetch", { id: string }, { title: string }>({
  key: "article.fetch",
  source: "../fixtures/ArticleView.vue",
  endpoint: ({ id }) => `/articles/${id}`,
});
await client.load(fetched, { id: "intro" });

// @ts-expect-error missing resource input
await client.load(article, {});
// @ts-expect-error optimistic updates must preserve the resource payload shape.
client.optimistic(article, { id: "intro" }, () => ({ title: 42 }));
// @ts-expect-error fetch resources preserve their endpoint input shape.
await client.load(fetched, { slug: "intro" });

defineDataResource({
  key: "broken",
  // @ts-expect-error source ownership is Vue-only
  source: "./broken.ts",
  loader: () => null,
});
