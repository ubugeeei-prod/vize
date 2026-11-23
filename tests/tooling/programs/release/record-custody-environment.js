process.stdout.write(
  JSON.stringify({
    token: Boolean(process.env.GITHUB_TOKEN),
    auth: Boolean(process.env.NPM_CONFIG__AUTH),
    secret: Boolean(process.env.example_secret),
    journal: process.env.VIZE_PUBLIC_NATIVE_CUSTODY,
  }),
);
