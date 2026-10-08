// Authored whole token controls for real comments versus regexp data/division.
export const semanticRegionControls: ReadonlyArray<
  readonly [string, string, ReadonlyArray<readonly [string, number]>]
> = [
  [
    "Member",
    "obj.return / 2 /* close */ + count",
    [
      ["obj", 8],
      ["return", 15],
      ["/", 21],
      ["2", 19],
      ["/* close */", 17],
      ["+", 21],
      ["count", 8],
    ],
  ],
  [
    "Control",
    "if(ready) /[/*]/.test(close) /* close */",
    [
      ["if", 15],
      ["ready", 8],
      ["/", 21],
      ["/", 21],
      ["*", 21],
      ["/", 21],
      ["test", 12],
      ["close", 8],
      ["/* close */", 17],
    ],
  ],
  [
    "Group",
    "(count) / 2 /* close */ + count",
    [
      ["count", 8],
      ["/", 21],
      ["2", 19],
      ["/* close */", 17],
      ["+", 21],
      ["count", 8],
    ],
  ],
];
