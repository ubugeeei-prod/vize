/** Reviewed whole loader programs from the actual release SOURCE trees.
 * These are not claims about historical public TGZ bytes. A future live
 * public-archive comparison must establish equality before any require(). */
export const nativeLoaderProgram = {
  sources: ["c2263205e7cf18ee0d738615285de073773bb513", "836b74b5aa2d877de5162e0515719c86a805df46"],
  modules: [
    {
      name: "index.js",
      bytes: 7407,
      sha256: "609cfb6924275f7055c57df84178bd597741b2d3d5ac8f3429ecfe88fad8cac3",
    },
    {
      name: "native-binding.js",
      bytes: 3106,
      sha256: "88a86f7d253bc4d08afb96d198feda5c8e0f4c1b5310ab1a3190ecc7a0eb46ba",
    },
    {
      name: "native-targets.js",
      bytes: 3771,
      sha256: "0c10b80b3f712c81f7fa3271de6dea12fc2e13cdf79c041e2a3b921b6cd16524",
    },
    {
      name: "native-binding-identity.js",
      bytes: 1705,
      sha256: "77174fd3b02313bd0cdd9ba781e4a87f74b4e22c1d386cfef775aa3fd81e7039",
    },
    {
      name: "language-projection.js",
      bytes: 7551,
      sha256: "9f2a8aa56033b86bbb51e25ec2f80f3d681e300ef547ef54d4731b0af90cbffe",
    },
  ],
} as const;
