# Changelog

All notable changes to this repository are tracked in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Conventional Commits](https://www.conventionalcommits.org/).

See [`docs/release/support-policy.md`](./docs/release/support-policy.md) for the
deprecation contract that backs the entries below.

## [Unreleased]

## [0.211.0] - 2026-06-13

### Fixed

- Isolate Deploy docs Musea example cargo checks from the shared sticky target cache (#1568)
- Skip cross-platform entrypoint assertion in fresh-install smoke (#506) ([9682241](https://github.com/ubugeeei/vize/commit/9682241bbc4eeaa4c8f40d18e041de16fd04963e))
- Install deps for coverage jobs ([27abd35](https://github.com/ubugeeei/vize/commit/27abd356046bd18ba5af02f1f08c0cb050f98904))
- Keep glob minimatch ranges compatible ([c1f88c6](https://github.com/ubugeeei/vize/commit/c1f88c69d903e30b9cd9fb8f5e329051dbb8688e))
- Stabilize oxlint plugin test output ([2d2c354](https://github.com/ubugeeei/vize/commit/2d2c354882762118803dbf4c3c32915b36fe7477))
- Provide tsx for package tests ([3a571cf](https://github.com/ubugeeei/vize/commit/3a571cfb18762147acff16269335f5e3f2062dcf))
- Run npm audit through vp ([93a4423](https://github.com/ubugeeei/vize/commit/93a4423d8f552966058ea126507b98c8332c4dc8))
- Stabilize recent issue batch ([aba27dd](https://github.com/ubugeeei/vize/commit/aba27dd9e8c1bbcc6db1cb96b2785ab7cc659609))

### Released

- Declare workspace MSRV (rust-version = 1.95.0) (#504) ([6674ad2](https://github.com/ubugeeei/vize/commit/6674ad2dcf31c157ad602cb180e0673d42b3019b))

## [0.105.0] - 2026-05-18

### Fixed

- Defer virtual vue runtime to vite (#486) ([7a54a9e](https://github.com/ubugeeei/vize/commit/7a54a9ee07bd76a25267d18645c23645abb3d791))
- Stabilize checks after main merges ([06513a0](https://github.com/ubugeeei/vize/commit/06513a09506674e9c2cc386ec3fd849823548d4b))
- Publish croquis cross-file crate (#484) ([ee27ae8](https://github.com/ubugeeei/vize/commit/ee27ae835591f0a3291fa8a0e624b847c79dcaaf))
- Declare Marketplace license (#483) ([58878d0](https://github.com/ubugeeei/vize/commit/58878d075eceb85fed42913fda56240526cd5d99))
- Declare public publish access (#482) ([91533ba](https://github.com/ubugeeei/vize/commit/91533ba94277884c923d0f52fb0be2155d20daff))
- Reject stale optional bindings (#481) ([cd311a1](https://github.com/ubugeeei/vize/commit/cd311a184d2880961890e68403c77eac645851ef))
- Await TypeScript config transforms (#480) ([6c9ba91](https://github.com/ubugeeei/vize/commit/6c9ba91cc410f2196ab60f1c4367386bbf6afed4))

## [0.104.0] - 2026-05-17

### Fixed

- Stabilize ci after main merges ([4a4b66e](https://github.com/ubugeeei/vize/commit/4a4b66ef95fd9bfa149c356a44761493c0148b68))
- Parse readonly dts members (#451) ([8106050](https://github.com/ubugeeei/vize/commit/810605068ca2a5c6c2ddc3bcdc11d623ebdce00c))
- Strip more template TypeScript expressions (#450) ([a94d37a](https://github.com/ubugeeei/vize/commit/a94d37a80fee8d8a0c7cbd8ffcc7cfe02bd9f432))
- Extract defaulted v-for destructure params (#449) ([faafbc8](https://github.com/ubugeeei/vize/commit/faafbc8b90b6b0d0bf62e3dee3a3075f7aa02186))
- Collect directive semantic token edges (#447) ([0510487](https://github.com/ubugeeei/vize/commit/051048725ec8ed52ae0e0518738e6365d5e5c3e5))
- Resolve destructured v-for aliases (#445) ([6812891](https://github.com/ubugeeei/vize/commit/6812891ddaaf2404e71589d5cc57859646de5973))
- Repair blog links (#446) ([591510e](https://github.com/ubugeeei/vize/commit/591510ea8a468c3a828e990c4445612f34086cf8))
- Close nested control-flow fixture gaps (#442) ([8a763f9](https://github.com/ubugeeei/vize/commit/8a763f9eceb135a0bfba213862b28b5f63359383))
- Format wasm package metadata ([993e917](https://github.com/ubugeeei/vize/commit/993e917d37f976464484ecebeabfe79202cd0d71))

### Performance

- Allocate comment buffer lazily (#454) ([0f6e017](https://github.com/ubugeeei/vize/commit/0f6e017897bf2dd1dcf33a3367f874effb902d98))
- Share sfc descriptor across rules (#453) ([948ee99](https://github.com/ubugeeei/vize/commit/948ee994e184f7cc345ec5ec85387e2dddd7e687))
- Scan virtual import paths without regex (#452) ([2d8fe7c](https://github.com/ubugeeei/vize/commit/2d8fe7c3249475dad8d5411d1745b6c6d19eea22))
- Skip unused virtual ts probes (#448) ([aeb4dba](https://github.com/ubugeeei/vize/commit/aeb4dba25f90a74b46a7ad73a292f75afcd31f3c))

## [0.103.0] - 2026-05-17

### Fixed

- Format package metadata ([aac8b99](https://github.com/ubugeeei/vize/commit/aac8b99c4be9f1c93480e7c47e5e0583ef08723b))
- Unblock v0.102 release checks (#443) ([ec9f1bc](https://github.com/ubugeeei/vize/commit/ec9f1bc5d243f3532ebe92e133dee63485992bd3))

### Performance

- Skip regular dependencies in Vite plugin hooks (#444) ([4ba524c](https://github.com/ubugeeei/vize/commit/4ba524c730a6078f6f600204a38b41816207da94))

## [0.102.0] - 2026-05-17

### Added

- Support alternate screen option (#400) ([b894dd4](https://github.com/ubugeeei/vize/commit/b894dd407b39c9d58cab8a1189ba014e0470e9e1))
- Add screen reader API parity (#372) ([3de9824](https://github.com/ubugeeei/vize/commit/3de9824c539adfcedc8a61a3a2a2ab70917005e9))
- Add telegraph report formats (#365) ([711338d](https://github.com/ubugeeei/vize/commit/711338d3a1b1576b75f037de4859447f19ba3368))
- Typecheck nuxt auto imports (#368) ([4130f33](https://github.com/ubugeeei/vize/commit/4130f33bffd1d17bb37160d49937c3c8983ff3dc))
- Add Ink-compatible APIs (#369) ([914a5cd](https://github.com/ubugeeei/vize/commit/914a5cdf3062117053c4ee82b7ad7bc6d2bf57a0))

### Documented release contract

- Add v1 alpha go-no-go checklist (#414) ([9aba147](https://github.com/ubugeeei/vize/commit/9aba147ecf86f4874cb51fbfdf1862df0cb56dc2))

### Fixed

- Escape generated static string literals (#440) ([c929185](https://github.com/ubugeeei/vize/commit/c929185b6f6a557d30d69671234c8c72c12cf120))
- Export ink instance types (#439) ([e50d78f](https://github.com/ubugeeei/vize/commit/e50d78f63e906ef69ffa0019fa53d2718a1e5863))
- Route cursor control through app context (#438) ([b53063d](https://github.com/ubugeeei/vize/commit/b53063da3aa81700507b95ecb2949f895252f2e5))
- Share animation scheduler (#437) ([c1943ad](https://github.com/ubugeeei/vize/commit/c1943ad2ab3e6cc5a36670a27cbca090918233dc))
- Persist static output above live frame (#436) ([874d4dd](https://github.com/ubugeeei/vize/commit/874d4dd769a71fa30bb9daf7b7baec96821cf916))
- Close built-in fixture gaps (#432) ([7a57bab](https://github.com/ubugeeei/vize/commit/7a57bab23e05429dc879cabe918ee34929332299))
- Coalesce block effects (#435) ([b42918f](https://github.com/ubugeeei/vize/commit/b42918f61c83afbfed8069b86d140e44d3d5187e))
- Coalesce block effects ([3d33d9d](https://github.com/ubugeeei/vize/commit/3d33d9d1b24e0a064d25d76e586233b1de1d8106))
- Publish vize before dependents (#430) ([d7d06ba](https://github.com/ubugeeei/vize/commit/d7d06ba03766c06f3733bdce0d83febbd10aa17f))
- Publish vize before dependents ([5441999](https://github.com/ubugeeei/vize/commit/54419999bfb067d25d14b56bb9bcd8a71413adcf))
- Restore destructured v-for keys (#428) ([0de7787](https://github.com/ubugeeei/vize/commit/0de7787aeac48a931e15bc590a3e3b803286dcb8))
- Restore destructured v-for keys ([14a4ff3](https://github.com/ubugeeei/vize/commit/14a4ff375bc1a31dac829da5c650347353130331))
- Constrain template semantic tokens (#427) ([116bd54](https://github.com/ubugeeei/vize/commit/116bd544022f2a27a035c5806ea533f4092ef11e))
- Constrain template semantic tokens ([8c90f99](https://github.com/ubugeeei/vize/commit/8c90f998183efb6aa6d00b36e1143c89ce56442a))
- Isolate renderToString streams (#422) ([18941ae](https://github.com/ubugeeei/vize/commit/18941ae7e4b81049f6ca4c81a2fa35b12a27f9d5))
- Fail coverage on fixture failures ([ad5135f](https://github.com/ubugeeei/vize/commit/ad5135ff8cd8afb20d3bca8e25691e6cb7389219))
- Isolate renderToString streams ([8cee93d](https://github.com/ubugeeei/vize/commit/8cee93dc243cff096e71b934716b1391e7c479e6))
- Preserve external output writes (#418) ([d4889e1](https://github.com/ubugeeei/vize/commit/d4889e189ecbe8a79b5ef500a0f62095aaba0811))
- Render final frame without terminal in ci (#412) ([1b37859](https://github.com/ubugeeei/vize/commit/1b3785920a5b7ce7bffa30c78d0178911f7b9896))
- Fail on compile errors (#406) ([d95bf71](https://github.com/ubugeeei/vize/commit/d95bf712b45b58aec0d236d94b0f5f4ea42ee9d7))
- Align platform binding versions (#392) ([6a4871c](https://github.com/ubugeeei/vize/commit/6a4871cf04bf0ca1d2228356ee059fe8bdeeb948))
- Point CLI installs at supported channels (#394) ([1989cae](https://github.com/ubugeeei/vize/commit/1989cae8f31e5ca445ed644729be3b0cb687c350))
- Align code action edits to UTF-16 (#398) ([4671a8c](https://github.com/ubugeeei/vize/commit/4671a8c0f9c4d0127aba18aa2a108f2e6152de6a))
- Route Rust serve through Vite (#410) ([802833a](https://github.com/ubugeeei/vize/commit/802833a916a4f8ddf04c9885d18cc6e71cd0c4cd))
- Protect text input composition state (#409) ([60a7201](https://github.com/ubugeeei/vize/commit/60a720124aeed836614219a0a7ae51b837f7ad85))
- Align tab input payload with ink (#411) ([025562c](https://github.com/ubugeeei/vize/commit/025562c66c51f1553220c79e644041cf38c2434b))
- Preserve Nuxt dev route modules (#417) ([1fc7db8](https://github.com/ubugeeei/vize/commit/1fc7db88fae44d743ad4937943e5b21cf7013a48))
- Generate valid Nuxt fallback stubs (#420) ([cd9d745](https://github.com/ubugeeei/vize/commit/cd9d74563fe980719f3e573cb69d8e58ddac260d))
- Detect non-interactive terminals (#405) ([bdd5161](https://github.com/ubugeeei/vize/commit/bdd5161990cc596c047b03b0d40535793d0d1ea6))
- Align Ink input paste handling (#393) ([56fd803](https://github.com/ubugeeei/vize/commit/56fd80332c6c8290262b06d810d371a2414abeaf))
- Version diagnostic publishes (#395) ([37d8e8e](https://github.com/ubugeeei/vize/commit/37d8e8e24e0a44ac251389d0a719d88477cd81f9))
- Reject invalid UTF-16 request positions (#374) ([26db417](https://github.com/ubugeeei/vize/commit/26db417d55ee75e9876ae65ec507df8ba5d1b296))
- Honor tsconfig declaration emit options (#373) ([81d32e7](https://github.com/ubugeeei/vize/commit/81d32e7063f82e6a7808fe03367022a7539d5ede))
- Correct UTF-16 rename ranges (#371) ([f0f2672](https://github.com/ubugeeei/vize/commit/f0f26721b61c7b84687bb103781a5c488912c2a9))
- Harden editor navigation and semantic tokens (#367) ([8b3deb2](https://github.com/ubugeeei/vize/commit/8b3deb24cc89ec63ce5273aeecc8827b39c67a5c))

## [0.98.0] - 2026-05-16

### Fixed

- Reduce Node heap retention (#363) ([8ae52d0](https://github.com/ubugeeei/vize/commit/8ae52d0be0497ede4c309fb31c1bb88f5e3f0ca1))

## [0.97.0] - 2026-05-16

### Added

- Track allocations and I/O in profile mode (#351) ([622e507](https://github.com/ubugeeei/vize/commit/622e50714c1ffbe7973b45a0ff8fa8564e8b6fcc))
- Tighten profile diagnostics (#348) ([041e57b](https://github.com/ubugeeei/vize/commit/041e57bf824155d70c62f79652e58751089d26c9))

### Fixed

- Avoid panic in napi batch result collection (#357) ([34f7862](https://github.com/ubugeeei/vize/commit/34f7862f203c34f23b1d2d3519a5e9b7b65f6ac8))
- Release build batch sources earlier (#358) ([d6fa997](https://github.com/ubugeeei/vize/commit/d6fa9977011548d2e4d28b4bf2a4ea1467cab12e))
- Avoid retaining check sources (#353) ([8b646c9](https://github.com/ubugeeei/vize/commit/8b646c985e9261d4044cab90cb288e6396409c2b))
- Recover unclosed comments (#352) ([afe86f7](https://github.com/ubugeeei/vize/commit/afe86f7d92ad136866d235153389d711cb53ccde))
- Reduce unsafe and panic paths (#347) ([731cce8](https://github.com/ubugeeei/vize/commit/731cce813a41207950923d89099ac2c87047c0b6))

### Performance

- Trim child traversal profile spans (#360) ([4f5838e](https://github.com/ubugeeei/vize/commit/4f5838eafb4d809be713f7a4b8b62eb1b8eabca3))
- Trim template traversal profile spans (#359) ([e68e40b](https://github.com/ubugeeei/vize/commit/e68e40b5c94071c0b7452c9847d1e0de8d655fca))
- Coalesce rule callback profiling (#355) ([0002c69](https://github.com/ubugeeei/vize/commit/0002c69b6b30863cc7f934d425bf66c6e325ab5c))

## [0.95.0] - 2026-05-16

### Fixed

- Clear MoonBit script warnings (#345) ([22be1f7](https://github.com/ubugeeei/vize/commit/22be1f7b89a773d5bd8eef3885e96fc8aabaa5ce))

## [0.94.0] - 2026-05-16

### Fixed

- Prefer workspace MoonBit toolchain (#344) ([c7e00dc](https://github.com/ubugeeei/vize/commit/c7e00dcc9eaf0dda5d623d7474257d23112a4e1c))
- Prefer workspace MoonBit toolchain ([10dc6b9](https://github.com/ubugeeei/vize/commit/10dc6b975d740e68408e6971fb43de6ad96a6a7f))
- Install matrix Rust targets (#343) ([9303c5a](https://github.com/ubugeeei/vize/commit/9303c5a0982bceaec74749986e614fb136fd3f2f))
- Install matrix rust targets ([5664037](https://github.com/ubugeeei/vize/commit/5664037edebe6846f3c40b14235a4a02e7b725c4))

## [0.93.0] - 2026-05-16

### Added

- Add opt-in ecosystem rules ([756c1b5](https://github.com/ubugeeei/vize/commit/756c1b59c86bc03407eea9c20a1f892b3d5f04d5))

### Fixed

- Forward vp run release arguments (#342) ([dab31d9](https://github.com/ubugeeei/vize/commit/dab31d96f99e44a86480ce31888addcf20d8b946))
- Forward task arguments ([06977ce](https://github.com/ubugeeei/vize/commit/06977ce81849f07a5acab51f3f7604775784cdf7))
- Forward task arguments ([bc4a30d](https://github.com/ubugeeei/vize/commit/bc4a30d42a8bcbe318ccec2b868ee27239d1ee84))
- Format blog navigation script ([a525956](https://github.com/ubugeeei/vize/commit/a5259565d1dc4e070d5562ce971436e718b374e0))
- Preserve destructured v-for bindings (#340) ([f72f386](https://github.com/ubugeeei/vize/commit/f72f386a9a2a0d895d2ae42ea0169d17741f9af0))
- Preserve destructured v-for bindings ([b10876e](https://github.com/ubugeeei/vize/commit/b10876e9137243be48f9405e44b88003c1906f49))
- Reject invalid LSP document positions (#337) ([4d44e7c](https://github.com/ubugeeei/vize/commit/4d44e7c165837b6011175fccb8f134c97df1eae8))
- Reject invalid LSP document positions ([4057c94](https://github.com/ubugeeei/vize/commit/4057c94c6d2792bbf37e95ccb49e2ce6989839c7))
- Ignore incomplete template type probes (#339) ([6bee9cb](https://github.com/ubugeeei/vize/commit/6bee9cb12ee25ba06b4afb223e186411d5a98d42))
- Ignore incomplete template type probes ([2afeaa4](https://github.com/ubugeeei/vize/commit/2afeaa4e063e10e6067d2b880b24f5e1143af415))
- Sanitize terminal output (#335) ([062b88a](https://github.com/ubugeeei/vize/commit/062b88a88a1de1e8ed86b25709ad8ab6d246e647))
- Count UTF-16 units in diagnostics (#334) ([693e5a3](https://github.com/ubugeeei/vize/commit/693e5a3954a23f3968e909676ba4d9f7725a372b))
- Preserve v-if template narrowing (#333) ([cd25c78](https://github.com/ubugeeei/vize/commit/cd25c7802352d6600b86200bc084473a8391b7ae))
- Preserve typed v-for aliases (#332) ([48255cc](https://github.com/ubugeeei/vize/commit/48255cc2d80c424508fa9ee2ca6146ad3e117f97))
- Handle UTF-16 document positions (#331) ([0f38177](https://github.com/ubugeeei/vize/commit/0f38177dd9ba5390c62a762c4e708805029d38d2))
- Collect event statement callees (#329) ([f957ca6](https://github.com/ubugeeei/vize/commit/f957ca660096160dd02f2287cb30d7dcc2ad66d9))
- Stop advertising unimplemented providers (#328) ([db60c95](https://github.com/ubugeeei/vize/commit/db60c95ba23161999b0ff5084a0748813ff4dee4))
- Catch dynamic props v-model mutations (#327) ([24ebf1e](https://github.com/ubugeeei/vize/commit/24ebf1eb748c5a12fc9c195f1b9ae6614d27ae59))
- Avoid local props v-model false positives (#326) ([4c1c93e](https://github.com/ubugeeei/vize/commit/4c1c93ee7525b48ed843fefd51d26fd1a129cb24))
- Probe computed template bindings (#325) ([994ae63](https://github.com/ubugeeei/vize/commit/994ae6314c7e3f2e3804b45fb956b1fff1e42cec))
- Surface sfc lint diagnostics (#324) ([df2044e](https://github.com/ubugeeei/vize/commit/df2044e5c2383838672f03c6bb4d9301a394d250))
- Catch nested prop v-model mutations (#323) ([c8a041c](https://github.com/ubugeeei/vize/commit/c8a041c44c2414d0f61caba529e40c2142081710))
- Catch computed event promise handlers (#322) ([c2cac77](https://github.com/ubugeeei/vize/commit/c2cac772c40ca0afde5feccf95316df3ec59c963))
- Catch optional event promise handlers (#319) ([26fa722](https://github.com/ubugeeei/vize/commit/26fa722d6dad807268f80e1117d1da16d820bacb))
- Catch optional event promise handlers ([23f89a9](https://github.com/ubugeeei/vize/commit/23f89a90b37d842ada3beda89a18e2123c3843ff))
- Catch member event promise handlers (#318) ([df8dada](https://github.com/ubugeeei/vize/commit/df8dadac88187bad4bd48a3b03c7570acd047dc8))
- Catch member event promise handlers ([a1fee61](https://github.com/ubugeeei/vize/commit/a1fee61d1433ad5d0883df597c95751135aabefd))
- Tighten floating promise handlers (#317) ([4e61c38](https://github.com/ubugeeei/vize/commit/4e61c3896f1790c9d6051b83b956c5970b89d534))
- Tighten floating promise handlers ([af80c05](https://github.com/ubugeeei/vize/commit/af80c0587d81378a1bee60641f4d9eaf9fe9940e))
- Normalize task shell locale (#315) ([aaaa7b0](https://github.com/ubugeeei/vize/commit/aaaa7b00f49da4a25031a4e9c285c5d57b9762bd))
- Normalize task shell locale ([02deb17](https://github.com/ubugeeei/vize/commit/02deb1736053c89e0d44385f9e2b31013c8f7446))
- Ignore regex literal identifiers (#314) ([d4237c1](https://github.com/ubugeeei/vize/commit/d4237c13e1674d45d50772b0ad9c5707cfbbc4ec))

## [0.92.0] - 2026-05-16

### Fixed

- Gate template binding completions (#313) ([1ad169f](https://github.com/ubugeeei/vize/commit/1ad169f31a64a731a0c3a23a28a182fea7645db0))
- Reduce lsp false positives (#310) ([24ef6e1](https://github.com/ubugeeei/vize/commit/24ef6e14324be9c7e2025b8f8317d1c1dbefdd9c))
- Recognize runtime emits validators (#309) ([8ee35b9](https://github.com/ubugeeei/vize/commit/8ee35b9dc26cf0fa45dab063be2021fddc35eb2c))
- Surface script parse diagnostics (#308) ([dadba8c](https://github.com/ubugeeei/vize/commit/dadba8c5df2ac90a3a1e982208d8e8ae899d77c6))
- Report broken sfc parse errors (#307) ([8cd3eab](https://github.com/ubugeeei/vize/commit/8cd3eabe8c7850daafcb16299542b610acbb7d01))

## [0.91.0] - 2026-05-16

### Added

- Migrate config loading to pkl (#93) ([a770c6c](https://github.com/ubugeeei/vize/commit/a770c6c45b0711b794c181076a2d92f163ec2d0a))

### Fixed

- Narrow unused component candidates (#301) ([b7ebb21](https://github.com/ubugeeei/vize/commit/b7ebb21b9d99ffd2cbce7ef4c460fc6dfd98de20))
- Report template parse errors (#290) ([a2b5a8d](https://github.com/ubugeeei/vize/commit/a2b5a8dc2190b09fbb45319ac32bfa12ea5270ff))
- Match options api component aliases (#303) ([f6f692c](https://github.com/ubugeeei/vize/commit/f6f692cad55299e155289c85964a17028c49c3f4))
- Allow v-model on dynamic component (#304) ([f623a79](https://github.com/ubugeeei/vize/commit/f623a794514477804c2865aeccd7133cafd7cd26))
- Skip custom component interactions (#300) ([43524c6](https://github.com/ubugeeei/vize/commit/43524c6d1546ed6b0936b5592ae58bf1c7c82ace))
- Ignore invalid change ranges (#299) ([ca31412](https://github.com/ubugeeei/vize/commit/ca314129da941092ade34fccc80341b40bd7e153))
- Clear stale virtual docs (#298) ([37d049a](https://github.com/ubugeeei/vize/commit/37d049a2e18a6e1431fd298563453e7602f385ee))
- Harden gallery URL escaping (#293) ([05f5dc5](https://github.com/ubugeeei/vize/commit/05f5dc5d2863bd8976d43f02f90c13715b405173))
- Harden fast template extraction (#292) ([ad7960f](https://github.com/ubugeeei/vize/commit/ad7960fa26c4b623efc4549f0e26b6712f0e598e))
- Suppress type diagnostics after parse errors (#286) ([2a2b08f](https://github.com/ubugeeei/vize/commit/2a2b08f909d13831025d2f6f7c1891969e3ad0fc))
- Remove release conflict markers (#284) ([3517f47](https://github.com/ubugeeei/vize/commit/3517f47210685c0899c7cb93e0118d00b5bfedfc))
- Catch event block floating promises (#278) ([066d77f](https://github.com/ubugeeei/vize/commit/066d77fce0c90be9cad9eb65c19224f353f9ce3d))
- Preserve encoded dev stylesheet assets (#276) ([51ad811](https://github.com/ubugeeei/vize/commit/51ad81160206f3c4a4d5345564da13cd3d865b8b))
- Normalize javascript anchor hrefs (#275) ([895d6df](https://github.com/ubugeeei/vize/commit/895d6df1661c9c2abdbbd5d97fe18579de0f184a))
- Skip regex literal browser names (#273) ([b91e449](https://github.com/ubugeeei/vize/commit/b91e4496aed6014da9c382cbc49f09e7c2e18827))

### Performance

- Reduce hot path overhead (#279) ([a0a5d7b](https://github.com/ubugeeei/vize/commit/a0a5d7bf02fb79ae2dbffc2a3f2aa63c63cc4423))
- Share template expression parse (#277) ([ec07160](https://github.com/ubugeeei/vize/commit/ec07160692ba07dd2124611c0f5b986402ad109a))

## [0.90.0] - 2026-05-15

### Fixed

- Allow direct typeof browser guards (#272) ([448dd6f](https://github.com/ubugeeei/vize/commit/448dd6f24cba2118bd65f434373d3f544f58f6e2))
- Source style metadata from native sfc (#271) ([db79594](https://github.com/ubugeeei/vize/commit/db79594371922abb1dc458b8a547efdf7e11556b))
- Flag static unsafe urls (#268) ([f6ccb0e](https://github.com/ubugeeei/vize/commit/f6ccb0e08a9d5a1024b7c07c2ad13f0989098cee))
- Support regexp aliases in virtual resolution (#267) ([6f0c925](https://github.com/ubugeeei/vize/commit/6f0c9258ca6eee414281ae8afa2e24b49f09144c))

### Performance

- Share template query walk (#270) ([8b819cb](https://github.com/ubugeeei/vize/commit/8b819cbf78dbb9dc796e9a21731f8968dc6f0753))

## [0.89.0] - 2026-05-15

### Fixed

- Resolve Nuxt virtual package aliases (#262) ([e52f5bc](https://github.com/ubugeeei/vize/commit/e52f5bcebf2ccf7a916465af696ab52638518a1d))
- Support css regexp aliases (#264) ([d8f492d](https://github.com/ubugeeei/vize/commit/d8f492d1824bcdc9838c342a5e17066a23d6aacc))

## [0.88.0] - 2026-05-15

### Fixed

- Detect nested template promises (#259) ([6374029](https://github.com/ubugeeei/vize/commit/63740292834a0123c5652de933b74780a581586e))
- Flag promises handled only by finally (#257) ([f2c345a](https://github.com/ubugeeei/vize/commit/f2c345a5e7039f055235ae2786bd1b63020aea73))
- Detect floating promises in control flow (#256) ([fe73fb9](https://github.com/ubugeeei/vize/commit/fe73fb98ef484335cb1e2292eabad1827c1d23e6))
- Honor bound literal input types (#255) ([bd6f865](https://github.com/ubugeeei/vize/commit/bd6f865e267fd4a637a9f486740538cec9aebdfe))
- Skip kebab-case built-ins (#254) ([967b9da](https://github.com/ubugeeei/vize/commit/967b9da3bb785a35e54bb01054a6b55850b257ff))
- Preserve block attributes (#253) ([165dde1](https://github.com/ubugeeei/vize/commit/165dde1b63a262afbae26feb1cd4add66d010f9b))
- Preserve default export modifiers (#252) ([f6a092a](https://github.com/ubugeeei/vize/commit/f6a092aac1474e25478d218567c81f8ed7c77841))

## [0.87.0] - 2026-05-15

### Fixed

- Avoid discarded oxc sourcemaps (#251) ([0e4074c](https://github.com/ubugeeei/vize/commit/0e4074ce6a2b3816375f1ddad71007f58a51a1eb))
- Honor package tsconfig field (#247) ([5237d05](https://github.com/ubugeeei/vize/commit/5237d0519b698726cd7834a19aaeb0ba0726e6b8))
- Guard css alias prefixes (#249) ([83b06aa](https://github.com/ubugeeei/vize/commit/83b06aadcdd3962b153597c90e95251c12a8d0b8))

## [0.86.0] - 2026-05-15

### Fixed

- Support tsconfig extends arrays (#245) ([987bc57](https://github.com/ubugeeei/vize/commit/987bc570d3acb6433ddee3b6fa9ef005c02aa9be))
- Preserve tsconfig path bases (#244) ([b54b3e4](https://github.com/ubugeeei/vize/commit/b54b3e42f6c2b51e1c0135a08b3cec515443a138))
- Stabilize regexp patterns (#241) ([eaba90d](https://github.com/ubugeeei/vize/commit/eaba90dbdd048636a7b9c54d49d7f8cf76c89805))

## [0.85.0] - 2026-05-15

### Fixed

- Resolve aliased vue before filtering (#243) ([ca30a65](https://github.com/ubugeeei/vize/commit/ca30a65d4b404fca08c0b4915fdda665165628dc))
- Filter quoted glob inputs (#242) ([6495de3](https://github.com/ubugeeei/vize/commit/6495de30cd11dcd3d9f342372c56563172cf7007))

## [0.84.0] - 2026-05-15

### Fixed

- Saturate source range offsets (#240) ([c3981f6](https://github.com/ubugeeei/vize/commit/c3981f6e96ba7cea0167de6671bbec7c7b234e6c))
- Preserve prototype-like token keys (#239) ([d2a5a41](https://github.com/ubugeeei/vize/commit/d2a5a4173901eda727e88a3a5b84d17068ab7431))
- Await type stripping transform (#237) ([58a8c8a](https://github.com/ubugeeei/vize/commit/58a8c8a11d0d32620ded3f52f2ef8675b9f1c195))

### Performance

- Scan offsets without slicing (#238) ([6e0b966](https://github.com/ubugeeei/vize/commit/6e0b9668f0f39333ae5c3bc1de9df74e9926cf49))

## [0.83.0] - 2026-05-15

### Fixed

- Isolate temporary config output (#234) ([756e8cd](https://github.com/ubugeeei/vize/commit/756e8cd3be1f6315cc68ad4296e38ef24463df1e))
- Normalize input rendering and paths (#233) ([2a2afbf](https://github.com/ubugeeei/vize/commit/2a2afbfb5aba0fddb117e0b2ee95fd89c3d9b48b))

## [0.82.0] - 2026-05-15

### Fixed

- Harden workflows and local dev surfaces (#232) ([d3d4f42](https://github.com/ubugeeei/vize/commit/d3d4f4261905b752215d12ce686b19652e08396f))

## [0.81.0] - 2026-05-15

### Fixed

- Use thread-local napi layout state ([1f08a8a](https://github.com/ubugeeei/vize/commit/1f08a8a93dc439a8eb8833e477ea665ba5cdeaf0))

## [0.79.0] - 2026-05-15

### Fixed

- Limit vue precompile memory usage (#229) ([28f9072](https://github.com/ubugeeei/vize/commit/28f9072a062be9ca83b8b25b3cb0cad196b89635))

## [0.78.0] - 2026-05-14

### Fixed

- Avoid compiling non-Vue macro imports (#224) ([a1ce44f](https://github.com/ubugeeei/vize/commit/a1ce44f591a2bef64298dced3b7b701f3ad9e668))

## [0.77.0] - 2026-05-13

### Added

- Harden cross-file and type-aware analysis (#215) ([6a205cc](https://github.com/ubugeeei/vize/commit/6a205cc3e5a52f6024d499015f3597d48be7c6d1))

### Fixed

- Restore mobile menu navigation (#216) ([74b509e](https://github.com/ubugeeei/vize/commit/74b509e0ad49914c20520ffd8d9f76e4c9b9e306))

## [0.76.0] - 2026-04-30

### Fixed

- Preserve imported definePage calls (#212) ([1320107](https://github.com/ubugeeei/vize/commit/132010792e1fc8458da501a634aaa4889e265c9f))

## [0.75.0] - 2026-04-27

### Added

- Support compile-time macro artifacts (#209) ([3579a2f](https://github.com/ubugeeei/vize/commit/3579a2f20946ab4cca92e6afbb91c1e96237fe0f))

## [0.74.0] - 2026-04-27

### Fixed

- Dark button contrast (#210) ([53e1622](https://github.com/ubugeeei/vize/commit/53e1622e71b7f8c335193689d913251f2b04e789))

## [0.73.0] - 2026-04-27

### Fixed

- Correct Vue prop source spans (#208) ([f5d043a](https://github.com/ubugeeei/vize/commit/f5d043a84ccefe91a0aaf1b9c292437c73cf38b4))

## [0.72.0] - 2026-04-26

### Fixed

- Refresh check-server vue overlays (#205) ([5af1c2d](https://github.com/ubugeeei/vize/commit/5af1c2d508718a2e7f01440f6f92f83428827071))

## [0.71.0] - 2026-04-26

### Fixed

- Improve e2e and tooling readiness (#204) ([3a87d25](https://github.com/ubugeeei/vize/commit/3a87d25f029f731be72705a19787743a996640eb))

## [0.70.0] - 2026-04-25

### Added

- Support declaration generation (#203) ([be036b5](https://github.com/ubugeeei/vize/commit/be036b5fc90d851ae8290a1746a9a4d3fcd3dd52))

### Performance

- Speed up typecheck virtual project (#202) ([22fd949](https://github.com/ubugeeei/vize/commit/22fd9490f4f95e7f78dcf3993d32a602f2c2ce97))

## [0.69.0] - 2026-04-24

### Performance

- Reduce props codegen scans (#197) ([4944bfd](https://github.com/ubugeeei/vize/commit/4944bfd62d083bfc1b82d94fa9331644be0645d2))

## [0.68.0] - 2026-04-24

### Fixed

- Resolve open issue regressions (#193) ([b7e4915](https://github.com/ubugeeei/vize/commit/b7e4915426a53ebb377faaba886c78fc8c6a0be4))

## [0.67.0] - 2026-04-24

### Performance

- Reduce compiler hot path allocations (#192) ([edf017e](https://github.com/ubugeeei/vize/commit/edf017e424dd3d04d712a37528eaa1861bf9082e))

## [0.66.0] - 2026-04-24

### Added

- Add npm ready and upgrade entrypoints (#190) ([0ef7b8d](https://github.com/ubugeeei/vize/commit/0ef7b8d61d68722b9e69cc731f8546d75492dabc))

## [0.65.0] - 2026-04-24

### Added

- Expose vize check command (#189) ([6b9de99](https://github.com/ubugeeei/vize/commit/6b9de9962d3f756217a6c60186b6d2754f56feda))

## [0.64.0] - 2026-04-24

### Added

- Implement HTML special tag parsing (RCDATA & RAWTEXT) (#187) ([b95fee6](https://github.com/ubugeeei/vize/commit/b95fee6d2d1aea5555d81fbd7d497e2f0d37c4c3))

### Fixed

- Harden SSR support for e2e fixtures (#188) ([674a26c](https://github.com/ubugeeei/vize/commit/674a26c7082581bfe10399c331cac1214aa35227))

## [0.63.0] - 2026-04-24

### Fixed

- Stabilize misskey v-model and infinite scroll regressions (#184) ([cabe029](https://github.com/ubugeeei/vize/commit/cabe0290d0bd7edd5f730eaeacf5f677f54664e9))

## [0.62.0] - 2026-04-23

### Added

- Enrich MCP workflows (#181) ([b7e187f](https://github.com/ubugeeei/vize/commit/b7e187ff67bd6c0bcbe2f84776ae446a8a68d773))

### Performance

- Tighten bench and profile workflows (#182) ([c98d0d0](https://github.com/ubugeeei/vize/commit/c98d0d08a27799a197287a3ca4666a5a14d16e53))

## [0.61.0] - 2026-04-23

### Fixed

- Guide users through extension setup (#179) ([3a6516d](https://github.com/ubugeeei/vize/commit/3a6516da3801720d4526babd137aa1be899bd44a))
- Vp build and remove build warnings (#180) ([6c6bf6b](https://github.com/ubugeeei/vize/commit/6c6bf6ba8e75cfad697f35dbb451a823fecba66c))

### Performance

- Speed up profiled SFC builds (#178) ([2a31855](https://github.com/ubugeeei/vize/commit/2a318553ad997dd9345fc3a05144eb2afee4d900))

## [0.60.0] - 2026-04-22

### Added

- Various renderers (#172) ([9bb1a0d](https://github.com/ubugeeei/vize/commit/9bb1a0df0a0c0819c7f7088434f6a08eed58fca2))
- Surface atelier output warnings ([d03e507](https://github.com/ubugeeei/vize/commit/d03e5076edc90f3ce6b6ec35d0d2c4e2f7d3bd0b))
- Expose vapor ssr template output ([ef0d041](https://github.com/ubugeeei/vize/commit/ef0d04113fb10160c1a3918f3a3589f6bf814dd5))

### Fixed

- Support ignored readmes ([3e9a69b](https://github.com/ubugeeei/vize/commit/3e9a69b184a3b2b0ea6a10fd46c82298b5f3572d))
- Skip ignored readmes ([32829dc](https://github.com/ubugeeei/vize/commit/32829dc8dad4d2ec1ecb9916e501922e6035f92c))
- Avoid async tabs in vapor app ([55d07ef](https://github.com/ubugeeei/vize/commit/55d07ef82db40d2667b99c62693ae06c007bcf28))
- Harden compiler macro regressions ([79e9e11](https://github.com/ubugeeei/vize/commit/79e9e110ba725255624cea545bcd6dd657e78348))
- Harden no-node-modules diagnostics ([a196265](https://github.com/ubugeeei/vize/commit/a196265bb40236b6aecacfd128c4e9094d9326db))
- Resolve dotted component bindings from setup ([5fe5165](https://github.com/ubugeeei/vize/commit/5fe51651dea8c38ee164ed6a955162758d7b034d))
- Avoid unicode panic in virtual ts generation ([2ac861d](https://github.com/ubugeeei/vize/commit/2ac861d0d2247fc4e715117468e4cf874b9ebd04))
- Warn on vapor ssr fallback ([754867a](https://github.com/ubugeeei/vize/commit/754867a31076397aa2a6ea9cb066d5f9e6da67d0))
- Normalize unocss vue.ts scan ids ([24ec332](https://github.com/ubugeeei/vize/commit/24ec332b577189c2433634c99eb6fb4cb0488b6d))
- Handle queried virtual module ids ([3c74810](https://github.com/ubugeeei/vize/commit/3c74810a20d678791ae450a2f88ffe741c0525d4))
- Preserve query strings on virtual vue ids ([fbb156b](https://github.com/ubugeeei/vize/commit/fbb156b5aca04da7cc5e6a5e7bbf287772a61128))

## [0.59.0] - 2026-04-22

### Fixed

- Artifact path (#173) ([1b103e3](https://github.com/ubugeeei/vize/commit/1b103e3846e997712b852899777d261a933e5416))
- Support custom renderer components across dom ssr and vapor ([a09d82c](https://github.com/ubugeeei/vize/commit/a09d82cf408aedcba34f0a245ea4cf8005fd8cd5))
- Harden production CSS extraction and WASM SFC downcompile ([6dbb607](https://github.com/ubugeeei/vize/commit/6dbb6077b134917913588236ceb31b41207f5eb2))
- Resolve lowercase component bindings across SSR and plugin ([e8d8484](https://github.com/ubugeeei/vize/commit/e8d8484247daec291b4f25bd202a7f81747f7f33))

## [0.58.0] - 2026-04-22

### Fixed

- Keep docs deploy checkout complete ([fe0b696](https://github.com/ubugeeei/vize/commit/fe0b69621694d0e71e9b6eba2732d48c44ec9c7c))
- Install moonbit before wasm CI builds ([63a74d2](https://github.com/ubugeeei/vize/commit/63a74d2eb53435fb5622394219f69562f2816873))

## [0.57.0] - 2026-04-22

### Added

- Moon scripts (#169) ([82c90ce](https://github.com/ubugeeei/vize/commit/82c90cef19bffe9fc56b613b4bad8cebde82a5bb))

### Fixed

- Make native artifact staging idempotent ([85679ed](https://github.com/ubugeeei/vize/commit/85679eda637af8555a628fbd6a38139bae50b9c6))
- Bundle fresco native release artifacts ([0f59360](https://github.com/ubugeeei/vize/commit/0f59360b85267e645a1dfcf0c72a7dfef72c728a))
- Harden npm release publishing ([7aa7ec1](https://github.com/ubugeeei/vize/commit/7aa7ec1a7bd5fe0cb867e52f9e6dfcac6079bb1c))
- Bypass napi cli for apple release builds ([f1fc6af](https://github.com/ubugeeei/vize/commit/f1fc6aff4583a1d43a0552ecc8865d41a652b7cc))
- Pin rust linker for windows releases ([2013a33](https://github.com/ubugeeei/vize/commit/2013a3387f93f49bc0381aad3e56d8f315617872))
- Run release helpers natively on windows ([0e239c6](https://github.com/ubugeeei/vize/commit/0e239c6ae62244954f1f30ee52077a06687a0065))
- Make clean_node_binaries target-agnostic ([1492c3b](https://github.com/ubugeeei/vize/commit/1492c3b2016010714a389ab51fd59e3fefb4c6dd))
- Derive windows moon helper target from matrix ([145846a](https://github.com/ubugeeei/vize/commit/145846a880a5a6d5b535621d98d21ce6abfd6af2))
- Run release helpers with js on windows ([b5c02f9](https://github.com/ubugeeei/vize/commit/b5c02f9597330fca3edc1ac448d3a8ff23b6188b))
- Expose moon shim to bash on windows ([4532e26](https://github.com/ubugeeei/vize/commit/4532e26f9d3fbc770fa4626f79e91d7e0b00577b))
- Harden release reruns on actions ([4910f27](https://github.com/ubugeeei/vize/commit/4910f2723d589f0f6600b3bd19426ea48b4751e4))
- Make release reruns idempotent ([6aaefb0](https://github.com/ubugeeei/vize/commit/6aaefb05d0f7acf13a85519260c0d908fe746a09))
- Harden github actions workflows ([2706577](https://github.com/ubugeeei/vize/commit/27065772e181f531e68c2de2155b6af8bfa9bcdd))

## [0.56.0] - 2026-04-21

### Fixed

- Avoid mapfile in release artifact collection ([5cb3b3c](https://github.com/ubugeeei/vize/commit/5cb3b3c03c68b1630d234fafc287a7f984875ecd))

## [0.53.0] - 2026-04-21

### Added

- Improve (#166) ([326225d](https://github.com/ubugeeei/vize/commit/326225da9825c6f8a2084ff3b6114520951f00ad))
- Syntax highlight (#168) ([0d5ff65](https://github.com/ubugeeei/vize/commit/0d5ff653f3f6d6785b18a949b74294bb8a94f8a3))

## [0.51.0] - 2026-04-21

### Fixed

- Musea (#114) ([538bd26](https://github.com/ubugeeei/vize/commit/538bd26c5dd1812db7137e46be5229c83a371b4f))

## [0.50.0] - 2026-04-21

### Added

- Improve ui (#164) ([706b65e](https://github.com/ubugeeei/vize/commit/706b65e399dbbc66e59a6099c297b030b92a70ae))
- Editor integ (#160) ([93ea30c](https://github.com/ubugeeei/vize/commit/93ea30c0c5ea99ab4ae154b98cb39a9e8dc64472))

### Fixed

- Ci (#162) ([2fb46e6](https://github.com/ubugeeei/vize/commit/2fb46e6a8f83755ce1f5a7269f1f9b43241d1501))

## [0.49.0] - 2026-04-20

### Added

- Oxlint plugin (#115) ([a56fe70](https://github.com/ubugeeei/vize/commit/a56fe704f3e2547fc88f6a4dcd5a241b413eb3d0))

## [0.48.0] - 2026-04-20

### Fixed

- Sanitize component asset identifiers (#158) ([add95cc](https://github.com/ubugeeei/vize/commit/add95cc9c5114ace519dade52dcdb18c24c2c3b9))

## [0.47.0] - 2026-04-20

### Added

- More compiler compat 20260419 (#157) ([2c2c530](https://github.com/ubugeeei/vize/commit/2c2c5304e9ce74e57374981b6f464713170dbde9))

## [0.46.0] - 2026-04-11

### Fixed

- Dependency resolution ([2dce32b](https://github.com/ubugeeei/vize/commit/2dce32b7e648591eb515383c35af6f657db38e1a))

## [0.45.0] - 2026-04-11

### Added

- Profiling mode ([440d548](https://github.com/ubugeeei/vize/commit/440d5484dc06cdfc1714a62dcb158075a8aecf29))

### Performance

- Tune compiler and benchmark profiling (#154) ([cf755a6](https://github.com/ubugeeei/vize/commit/cf755a6010c708f6e26336ef3bf0eed799c2b154))

## [0.44.0] - 2026-04-11

### Added

- Improve corsa-backed type checking (#149) ([a1188bb](https://github.com/ubugeeei/vize/commit/a1188bb51519f094612f02b15ca979ae1ef91bc5))

## [0.42.0] - 2026-04-08

### Added

- Configuration (#145) ([ca35296](https://github.com/ubugeeei/vize/commit/ca352968825dca774dbd1a8d3fb5aaeaea38cb0d))

### Fixed

- Use AST spans for script setup sections (#141) ([4bffbdc](https://github.com/ubugeeei/vize/commit/4bffbdc3984b2817bc808a6a3608fa8e0071c90f))
- Ignore code in comments (#140) ([bc630ad](https://github.com/ubugeeei/vize/commit/bc630ad69ab2068b169e7f247990f31ba2abeeaa))

## [0.41.0] - 2026-04-06

### Fixed

- Restore CI toolchain compatibility ([a40122d](https://github.com/ubugeeei/vize/commit/a40122db4f0ac86d71bc8d4d5c2601c24a31e0a6))

## [0.40.0] - 2026-04-06

### Added

- Resolve @import recursively via LightningCSS (#124) ([0ed8583](https://github.com/ubugeeei/vize/commit/0ed8583561923143e46d6aa98c17626637568fca))
- Accumulate decoded HTML entities in value buffer (#134) ([885e2f8](https://github.com/ubugeeei/vize/commit/885e2f80c7c3eaab07a618867489a9e737465f64))
- Migrate to corsa-bind (#135) ([56ee67a](https://github.com/ubugeeei/vize/commit/56ee67a7031eb2f3d00b711525e4ac2fe3fe0822))

### Fixed

- Restore failing CI checks (#138) ([0f8806a](https://github.com/ubugeeei/vize/commit/0f8806a384527d267be64d2d50167831076968fc))
- Check the range valid for errors (#137) ([f6c3158](https://github.com/ubugeeei/vize/commit/f6c31580ae9f6aab796f1b053e651ed6e012dbf2))

## [0.39.0] - 2026-03-28

### Fixed

- Align filenames with wasm-bindgen output (#133) ([e899751](https://github.com/ubugeeei/vize/commit/e899751d46bc80a554573b734c5cfd5ad6183a2d))

## [0.38.0] - 2026-03-26

### Added

- HTML entity decoding in tokenizer (#129) ([d0eef27](https://github.com/ubugeeei/vize/commit/d0eef274d292eaabb6adb3d7ef1e8fbe59c6b718))

### Fixed

- Improve Vue SFC TypeScript handling across compiler and integrations (#128) ([7f2aecf](https://github.com/ubugeeei/vize/commit/7f2aecfd9b6b9e742b083515b487ff0be90a952e))
- Parse named style module attrs (#127) ([ce1b830](https://github.com/ubugeeei/vize/commit/ce1b8309f0de9f9b176ebeea1ee596d76bbcc4e1))

## [0.37.0] - 2026-03-19

### Fixed

- Publishing ([5b0eb52](https://github.com/ubugeeei/vize/commit/5b0eb522f43e89c3727e8a39e1e3e393087759c8))

## [0.36.0] - 2026-03-19

### Added

- Separate rule path (#125) ([26f8aca](https://github.com/ubugeeei/vize/commit/26f8aca1b428ba78d322b3595e07e0a11f662f7c))

## [0.34.0] - 2026-03-18

### Added

- Add native scoped css support and refactor style loading chain (#119) ([bf79676](https://github.com/ubugeeei/vize/commit/bf7967649e4d00f0c5512293c352e220c2cab3c2))
- Css modules support to compile_css via lightning css (#123) ([3340e5b](https://github.com/ubugeeei/vize/commit/3340e5b61a8d3ef4717222d76e68f9cdf93a26fa))

## [0.30.0] - 2026-03-13

### Added

- More stable, more performant (#117) ([de4e0bd](https://github.com/ubugeeei/vize/commit/de4e0bd673bafa7b82e5f3df3075f1dd67e21cc4))

## [0.28.0] - 2026-03-11

### Added

- E2e utils and fix more (#112) ([fbf6d1a](https://github.com/ubugeeei/vize/commit/fbf6d1af460c6794cf29007c6b1c6900e74b752f))

## [0.25.0] - 2026-03-08

### Added

- Other build tools (#111) ([3627c1d](https://github.com/ubugeeei/vize/commit/3627c1d7e9a179e5d562e616edd73570b31b977b))
- Add @vizejs/rspack-plugin (#94) ([4a2c841](https://github.com/ubugeeei/vize/commit/4a2c841b912a8411777c9da2a4e6fa548a257297))
- Add .gitattributes (#110) ([0d180e6](https://github.com/ubugeeei/vize/commit/0d180e654cbf794c3bbed8907b625c543991916a))

## [0.24.0] - 2026-03-08

### Added

- Use vapor compiler (#109) ([41be4b1](https://github.com/ubugeeei/vize/commit/41be4b137e6fab0aeabd1312f57229193980bffe))

## [0.23.0] - 2026-03-07

### Added

- More compatibility (20260307) (#98) ([93b93cb](https://github.com/ubugeeei/vize/commit/93b93cb954810568ab2d39292fc72faeb2a42037))

## [0.22.0] - 2026-03-07

### Added

- More vapor compat (#96) ([7ef26cc](https://github.com/ubugeeei/vize/commit/7ef26cc0b838ddf5db1e01ec749325c21ab9fe80))

## [0.21.0] - 2026-03-06

### Added

- Autocomplete and typecheck (#95) ([399c179](https://github.com/ubugeeei/vize/commit/399c1798200f2ccf8a8db47c5095c1a617d97670))

### Fixed

- V-for class and style binding (#97) ([572e9bc](https://github.com/ubugeeei/vize/commit/572e9bcae38e5a4f3b4375b78ae3c00a89112cdd))

## [0.20.0] - 2026-03-04

### Added

- More typechecker (#92) ([70803b9](https://github.com/ubugeeei/vize/commit/70803b980fe95919c028381fb9e83e30b7d4dc49))

## [0.17.0] - 2026-03-03

### Added

- More compatibility (20260302) (#90) ([b0b7f06](https://github.com/ubugeeei/vize/commit/b0b7f0600d5dc2ee67885e5ddab59a6fbafccbe0))

## [0.16.0] - 2026-03-01

### Added

- More compatibility ([b0807e3](https://github.com/ubugeeei/vize/commit/b0807e3e32d642db070895e63e1d8064792ae556))

### Fixed

- Ci ([913db90](https://github.com/ubugeeei/vize/commit/913db904a8d6d8da39d85209d782d625aaaf7264))

## [0.15.0] - 2026-03-01

### Fixed

- @vize:forget comment causing undefined VNode children (#88) ([b5b6f68](https://github.com/ubugeeei/vize/commit/b5b6f68fa4a43fdb1d23a0e9d9f7d967466c0806))
- Install browser on ci (#87) ([161233b](https://github.com/ubugeeei/vize/commit/161233b2a957f5294721c840eebfeb99564d3662))

## [0.14.0] - 2026-03-01

### Added

- Unify lint suppression under @vize:forget and fix playground warnings (#84) ([3db526d](https://github.com/ubugeeei/vize/commit/3db526df6d6cf49978f141a731046839a030c8bc))

## [0.13.0] - 2026-03-01

### Fixed

- Misskey parse error (#85) ([7496807](https://github.com/ubugeeei/vize/commit/7496807db91087009eafff88432ff3f0ed3717cd))
- Playground xfile and musea example vrt (#83) ([62681ab](https://github.com/ubugeeei/vize/commit/62681ab4309485cb2c25216522678cdfe100fea4))

## [0.11.0] - 2026-02-28

### Fixed

- Add `.value` for SetupRef assignments and handle multi-statement event handlers (#81) ([6792e38](https://github.com/ubugeeei/vize/commit/6792e383a69ee09d8895abbbd43ae06a1603e766))

## [0.9.0-alpha] - 2026-02-27

### Added

- More compatibility (#74) ([8551b4d](https://github.com/ubugeeei/vize/commit/8551b4d455f60fd4630c2dd775177603dcd94499))

### Fixed

- Unexpected itaric and text transform (#78) ([8bffa50](https://github.com/ubugeeei/vize/commit/8bffa50547a622fdc480941cd7a860f2953a5df2))

## [0.9.0] - 2026-02-23

### Added

- Props controll (#77) ([203d24f](https://github.com/ubugeeei/vize/commit/203d24fdad7fe99886daca819c6ae207e8221edd))

## [0.4.0] - 2026-02-22

### Added

- Formatter (#72) ([06a5c56](https://github.com/ubugeeei/vize/commit/06a5c56da9b8ecefd472e46ad66c8dd9d27c1bf4))

### Fixed

- Windows build ([8656c37](https://github.com/ubugeeei/vize/commit/8656c37b8749a00a4b15237b2d0dc34b93c74bca))

## [0.3.0] - 2026-02-22

### Added

- New lint rules, diagnostic render layer, clippy fixes (#73) ([aec4951](https://github.com/ubugeeei/vize/commit/aec4951f99dd0a324cc823f354daa997322b5e2c))

## [0.2.0] - 2026-02-22

### Added

- Comfigure og ([463b0bf](https://github.com/ubugeeei/vize/commit/463b0bff63e83054309315e3d9edaa467cce13e2))
- Comfigure og ([63ba8bc](https://github.com/ubugeeei/vize/commit/63ba8bcbebe2e2ad5118be0c2acedda6dd2717aa))

### Fixed

- Og ([827adc6](https://github.com/ubugeeei/vize/commit/827adc68db4827ec0eb0b2104ad559c5340efd89))

## [0.0.1-alpha.121] - 2026-02-21

### Added

- Rebrand (#69) ([bc4fe87](https://github.com/ubugeeei/vize/commit/bc4fe87ef7dd918271819e368c871825e7fa899d))
- New brand (#68) ([593345c](https://github.com/ubugeeei/vize/commit/593345ca0e73d86d5da4f66e44a3f8b7d6681bba))

## [0.0.1-alpha.114] - 2026-02-20

### Performance

- Musea a11y test ([2e9d12f](https://github.com/ubugeeei/vize/commit/2e9d12f32862954a48b6b2a304e4f9e4aa2bc811))

## [0.0.1-alpha.113] - 2026-02-20

### Added

- Improve ui (#67) ([b2eda71](https://github.com/ubugeeei/vize/commit/b2eda71acbc70538045a0d4e5e563dc55f9e6e58))

### Fixed

- TS error in Input.vue event target type cast ([83ba4a1](https://github.com/ubugeeei/vize/commit/83ba4a1394039f031a9f2e2a329f86bb4313e36b))

## [0.0.1-alpha.112] - 2026-02-20

### Fixed

- Prevent release script from breaking lockfile ([5dbdad9](https://github.com/ubugeeei/vize/commit/5dbdad93a62e27bee9588343fe5b061f73474c49))

## [0.0.1-alpha.111] - 2026-02-20

### Fixed

- Lock ([ef71a0d](https://github.com/ubugeeei/vize/commit/ef71a0dbebcd19fdc289229d7cea11e463b8ba45))

## [0.0.1-alpha.110] - 2026-02-20

### Fixed

- Revert cli optionalDependencies to 0.0.1-alpha.108 ([e03f400](https://github.com/ubugeeei/vize/commit/e03f400cb0da87d27415b025ca2fc9e9c1d059be))
- Update pnpm-lock.yaml for v0.0.1-alpha.109 ([6ec58aa](https://github.com/ubugeeei/vize/commit/6ec58aa35f317c54b0a9ce7b081229a7ff38fa44))

## [0.0.1-alpha.109] - 2026-02-20

### Added

- Improve ui (#65) ([9873ff2](https://github.com/ubugeeei/vize/commit/9873ff295be03731c3c623e8b0ac1b3b6b8c7e6f))

### Fixed

- Npm cli path (#66) ([ec3e591](https://github.com/ubugeeei/vize/commit/ec3e591aa231624a006010219534d05450387588))

## [0.0.1-alpha.108] - 2026-02-20

### Added

- Comment directive (@vize) (#64) ([708a0fa](https://github.com/ubugeeei/vize/commit/708a0fade9bd555ef3020486a606aa5f2b865d68))

## [0.0.1-alpha.107] - 2026-02-20

### Fixed

- Cli install (#63) ([b042a63](https://github.com/ubugeeei/vize/commit/b042a6328ae1da474425766891aeb8243c09a16e))

## [0.0.1-alpha.106] - 2026-02-20

### Fixed

- Npm cli ([643cfef](https://github.com/ubugeeei/vize/commit/643cfefbe191ace6ff4cd6f9f6eba12c57479827))

## [0.0.1-alpha.105] - 2026-02-20

### Added

- Nuxt compiler option ([28c979d](https://github.com/ubugeeei/vize/commit/28c979df219b96d95b651d88c4afe558f50cf353))

## [0.0.1-alpha.103] - 2026-02-20

### Added

- Delegate some bugs from vuefes (#62) ([b6031f3](https://github.com/ubugeeei/vize/commit/b6031f342d0f5db043bf87a70a18e8de2c215eb5))

## [0.0.1-alpha.102] - 2026-02-16

### Added

- More compiler compatibility (thanks to ushironoko (2)) (#61) ([5487655](https://github.com/ubugeeei/vize/commit/54876556d2742538634e0a085f696f69ac21b9b1))

## [0.0.1-alpha.101] - 2026-02-10

### Fixed

- Fix matchGlob breaking \*_ pattern with _ replacement ([fdce650](https://github.com/ubugeeei/vize/commit/fdce650dffbccda26c3c549b241b8391e466aa21))

## [0.0.1-alpha.100] - 2026-02-09

### Fixed

- Resolutions ([33d79ad](https://github.com/ubugeeei/vize/commit/33d79ad3b2ed717885c194f2b1bc002e02fcbb08))
- Remove provenance from publishConfig for local publish support ([da56b74](https://github.com/ubugeeei/vize/commit/da56b744a17dacf085f48abb22cc75509d8baf31))
- Skip subpath imports (#) in resolveId ([bedaf67](https://github.com/ubugeeei/vize/commit/bedaf67d2efb99c300fc0efa25dbce7ca567f9dd))

## [0.0.1-alpha.99] - 2026-02-09

### Fixed

- Update release script to handle fresco-native platform packages ([731c1e5](https://github.com/ubugeeei/vize/commit/731c1e5a751912295de0871fb4822b2a6fa438d7))

## [0.0.1-alpha.98] - 2026-02-09

### Fixed

- Remove stale optionalDependencies from fresco-native ([a5d6123](https://github.com/ubugeeei/vize/commit/a5d61230aa66cf86ab21fdc6db47dfb08f5723ee))

## [0.0.1-alpha.94] - 2026-02-09

### Added

- Add @vizejs/nuxt all-in-one package for Nuxt integration ([8c6c3de](https://github.com/ubugeeei/vize/commit/8c6c3de464e54ff9ba99d36dcf9d2c4f4f8139da))

## [0.0.1-alpha.87] - 2026-02-09

### Added

- Add new typechecker diagnostics with linter separation (#58) ([3abe937](https://github.com/ubugeeei/vize/commit/3abe93712be14dddc77c247b95fc82e6815838a8))

## [0.0.1-alpha.85] - 2026-02-09

### Fixed

- Emit ts (preserve) (#56) ([45b9e92](https://github.com/ubugeeei/vize/commit/45b9e92a1f507c4d42f2eabdf1f71dbeb17bd470))

## [0.0.1-alpha.84] - 2026-02-09

### Added

- Token management (#53) ([e292739](https://github.com/ubugeeei/vize/commit/e292739f21bec4ea79feadb0be8abbcb506bf977))

## [0.0.1-alpha.83] - 2026-02-09

### Added

- More compat (#55) ([6430d2d](https://github.com/ubugeeei/vize/commit/6430d2d3b54df892058ce6c2fb8ad3b582cb85fa))
- Improve ui (#54) ([4ef4a3a](https://github.com/ubugeeei/vize/commit/4ef4a3afafe04a534144547ea2ceb3a19ee67614))

## [0.0.1-alpha.82] - 2026-02-08

### Added

- More (#51) ([68bff6f](https://github.com/ubugeeei/vize/commit/68bff6ffac92cd614aee0d2f7f2bbea7deb3dee0))

## [0.0.1-alpha.81] - 2026-02-08

### Added

- Use Croquis as compiler infrastructure with ReactivityTracker (#52) ([31b2205](https://github.com/ubugeeei/vize/commit/31b22053a889ca0e0c1a844b90079d20c136db08))

## [0.0.1-alpha.79] - 2026-02-08

### Fixed

- Ui (#50) ([e2b9bfb](https://github.com/ubugeeei/vize/commit/e2b9bfb1d8ac17f07ad72203c468295e0420ab9c))
- Scope visualiztion (#49) ([ce02677](https://github.com/ubugeeei/vize/commit/ce02677122928bcfd28ef6863b36900acb770787))

## [0.0.1-alpha.78] - 2026-02-08

### Added

- More compiler compatibility (#43) ([c7a000a](https://github.com/ubugeeei/vize/commit/c7a000ab7257d0f8052eef8b31587c63d315bb02))

## [0.0.1-alpha.77] - 2026-02-08

### Added

- Config (#48) ([f263d86](https://github.com/ubugeeei/vize/commit/f263d860d6af4667aa2e5ff98b01d1ae6337a018))

## [0.0.1-alpha.76] - 2026-02-07

### Added

- Improve type checker (#46) ([9d34dfb](https://github.com/ubugeeei/vize/commit/9d34dfb1a20fe6fe1f2d4f3b8e2619bbfa8cce2d))

## [0.0.1-alpha.75] - 2026-02-07

### Added

- Enrich musea (#44) ([b91be69](https://github.com/ubugeeei/vize/commit/b91be691e334d5cb198100b762568d44845a0fb1))

## [0.0.1-alpha.74] - 2026-02-07

### Added

- Improve type checker (#45) ([5d38fdc](https://github.com/ubugeeei/vize/commit/5d38fdca1c53c2ab496779f8bb8a66035d227a16))

## [0.0.1-alpha.73] - 2026-02-06

### Added

- Improve linter (#42) ([96d00ed](https://github.com/ubugeeei/vize/commit/96d00edf6498572f8bdacfabdcf750dd171c6c34))

## [0.0.1-alpha.72] - 2026-02-06

### Fixed

- Resolve vize virtual ([eb9b5de](https://github.com/ubugeeei/vize/commit/eb9b5de96decb52f35091bea41ed149cc795f444))

## [0.0.1-alpha.66] - 2026-02-06

### Fixed

- Ignore gitignored READMEs in release script ([86de003](https://github.com/ubugeeei/vize/commit/86de003e1dd562e55c7994eb2ad87c6a8380286e))
- Publishing ([d1ad380](https://github.com/ubugeeei/vize/commit/d1ad380a48988c1b9a44577a4992f217bccf30bb))
- Publishing ([f694295](https://github.com/ubugeeei/vize/commit/f6942958cfa80700f7e05d7c12bcf6521e722905))
- Publishing ([0aec21d](https://github.com/ubugeeei/vize/commit/0aec21d94bd5924e47db85582d8ca4458a8d9a85))
- Publishing ([a6a5ea1](https://github.com/ubugeeei/vize/commit/a6a5ea1e839c0fa6a99c778455e25db374e16d2a))
- Publishing ([8ce6ba9](https://github.com/ubugeeei/vize/commit/8ce6ba9c5f6edef5c6667ba89db37eec67e3ebc8))
- Publishing ([6d2cda0](https://github.com/ubugeeei/vize/commit/6d2cda0461cc6ef5627a76323ea07748140f2fea))
- Publishing ([ae1d451](https://github.com/ubugeeei/vize/commit/ae1d4518f2390c19a3a83445468663387c7999e8))
- Publishing ([7fe4626](https://github.com/ubugeeei/vize/commit/7fe4626c26c39a42d46cfaa552dcbec0321e0d17))
- Publishing ([2a43e77](https://github.com/ubugeeei/vize/commit/2a43e7764e0b20737caa3c99a9fe62c22f9ee344))

## [0.0.1-alpha.56] - 2026-02-06

### Fixed

- Ci ([5a14390](https://github.com/ubugeeei/vize/commit/5a14390e816b8ec44339d55439d7b3f0247df6e3))

## [0.0.1-alpha.55] - 2026-02-06

### Fixed

- Package name ([46be92a](https://github.com/ubugeeei/vize/commit/46be92a1ec230c40cfc94996830669d6a889cb43))

## [0.0.1-alpha.53] - 2026-02-06

### Performance

- Ci ([9918b30](https://github.com/ubugeeei/vize/commit/9918b301c3944c99e0e4d0fc76e1a6904d84c74d))

## [0.0.1-alpha.51] - 2026-02-06

### Fixed

- Publishing ([d70d52f](https://github.com/ubugeeei/vize/commit/d70d52f1c463a40f70af8cdd208380d1d670746f))
- Publishing ([2202cb7](https://github.com/ubugeeei/vize/commit/2202cb746cb19a0e44950b4a8ef8060c64795b6b))

## [0.0.1-alpha.50] - 2026-02-06

### Fixed

- Publishing ([ac4cadd](https://github.com/ubugeeei/vize/commit/ac4cadd7324e014aa118837065f2300aacb87d4b))

## [0.0.1-alpha.49] - 2026-02-06

### Fixed

- Publishing ([58d54d8](https://github.com/ubugeeei/vize/commit/58d54d8769c8e16c88b3b7c1e1488532aebc00c5))

## [0.0.1-alpha.46] - 2026-02-06

### Fixed

- Publishing ([d578a96](https://github.com/ubugeeei/vize/commit/d578a9637dcbb1b483673cb78da62e219289d487))

## [0.0.1-alpha.45] - 2026-02-06

### Fixed

- Publishing ([656e8c4](https://github.com/ubugeeei/vize/commit/656e8c4590204237b8753697cf4632f5a62c28bb))

## [0.0.1-alpha.44] - 2026-02-06

### Fixed

- Publishing ([b2d61c1](https://github.com/ubugeeei/vize/commit/b2d61c1d23b81cea9f2c9bd3b67e207d5a9433c4))

## [0.0.1-alpha.38] - 2026-02-05

### Fixed

- Publishing ([efc2f20](https://github.com/ubugeeei/vize/commit/efc2f2005e0a4cd8ced65a9200e94300443d0c0a))

## [0.0.1-alpha.37] - 2026-02-05

### Fixed

- Publishing ([c4223e0](https://github.com/ubugeeei/vize/commit/c4223e02e37d8c83fbd9c9fcdb3574084cde7ee9))

## [0.0.1-alpha.36] - 2026-02-05

### Fixed

- Publishing ([b312920](https://github.com/ubugeeei/vize/commit/b3129203ba679b621fffd917131357b53d9f0726))

## [0.0.1-alpha.35] - 2026-02-05

### Fixed

- Publishing ([e89fa81](https://github.com/ubugeeei/vize/commit/e89fa81abf625518c8c91482385c21575c4270b1))

## [0.0.1-alpha.34] - 2026-02-05

### Fixed

- Publishing ([ccd295d](https://github.com/ubugeeei/vize/commit/ccd295d9cc3e6e21a7e0d64adb618e43f223cbd3))

## [0.0.1-alpha.33] - 2026-02-05

### Fixed

- Publishing ([56956bd](https://github.com/ubugeeei/vize/commit/56956bd3b4192e2892edad775fd26e0ca1954c9d))

## [0.0.1-alpha.32] - 2026-02-05

### Added

- Self host playground (#32) ([95a74c2](https://github.com/ubugeeei/vize/commit/95a74c2ead7e22349d70d1083cd03b98937b565e))
- Ssr (#31) ([432c328](https://github.com/ubugeeei/vize/commit/432c328df5411540666567fa6598b4d5c38a5c02))
- Support tui target ([0c99c79](https://github.com/ubugeeei/vize/commit/0c99c79c1a070624414c13af088488f80563f835))
- Support tui target ([7682e42](https://github.com/ubugeeei/vize/commit/7682e42594c79c6c4ac8cf86cf6208df591fae94))
- Improve checker and ide support (#30) ([3f4273a](https://github.com/ubugeeei/vize/commit/3f4273ae6dbe5355e48e9dedcf2badc39949a21b))
- Unique id (#29) ([943fea1](https://github.com/ubugeeei/vize/commit/943fea14b0d0f8acf28772699779ba2b72c8caf7))
- Valid aria role (#28) ([d838211](https://github.com/ubugeeei/vize/commit/d8382113f01ba08fe43774db3f4d028759878a94))
- Valid aria props (#27) ([a35117b](https://github.com/ubugeeei/vize/commit/a35117b28ca27f6b0d9fcfe25c8e5912b8e6f5ca))
- Some aliasing ([337219c](https://github.com/ubugeeei/vize/commit/337219ca7fac7ededb0075549841937b24b0a24a))
- Cross file analyze (#4) ([7c1337b](https://github.com/ubugeeei/vize/commit/7c1337b43fc038db8ed50785cd7ff6cd8c2999a2))
- Tsgo (#3) ([ee5eb1b](https://github.com/ubugeeei/vize/commit/ee5eb1b4a579d5e54df94873d383810f9ac84370))
- More rich, more performant ([08eb760](https://github.com/ubugeeei/vize/commit/08eb760bae05426251031bd9b6523edae23e7b2f))
- Improvement ([4a96edf](https://github.com/ubugeeei/vize/commit/4a96edf49cc815971baeac6cf657bf710f074e86))
- Improvement ([e5cc595](https://github.com/ubugeeei/vize/commit/e5cc595c8bcdd794ea64bf5b6623c3c1a838d4d4))
- Move typecheck core to canon crate with proper Virtual TypeScript generation ([8537e34](https://github.com/ubugeeei/vize/commit/8537e348e9f136e30a7c6393eedb0c010e58d837))
- Improvement ([0751c54](https://github.com/ubugeeei/vize/commit/0751c54f9bd357f9b8d31c2d71e6aa3fc6c3aade))
- Improvement ([f33084c](https://github.com/ubugeeei/vize/commit/f33084ccecf94ddac05179f75ae3b294ffbec133))
- Improvement ([78d5541](https://github.com/ubugeeei/vize/commit/78d554103ecb0d56d726f26f3d801b7471f9c455))
- Improvement ([122e595](https://github.com/ubugeeei/vize/commit/122e595ced68a6375a96ddc62213ec996f837539))
- Improvement ([a21107d](https://github.com/ubugeeei/vize/commit/a21107d25e4f37546ed43a9bfcba1fc4303cbc7c))
- Improvement ([42d93b6](https://github.com/ubugeeei/vize/commit/42d93b62388e87a17377bd4592c1a19511b0e0ad))
- Croquis ([0f97955](https://github.com/ubugeeei/vize/commit/0f97955c02095dfcdee9f9c86742bc07a8313fed))
- Add vize_croquis crate for semantic analysis ([cd4f757](https://github.com/ubugeeei/vize/commit/cd4f75735ce131e2c31105c5ac3b04eb2810028a))

### Fixed

- Improve sfc compiler compatibility (#39) ([a303a49](https://github.com/ubugeeei/vize/commit/a303a49c86ef410093ac0b69231e463c7fd9d325))
- Type strip and multiline attrs (#37) ([39a98b8](https://github.com/ubugeeei/vize/commit/39a98b823cb6b38f6209bde45bf651e4ca230ad5))
- Correct @vizejs/wasm package name for npm publishing ([46158e0](https://github.com/ubugeeei/vize/commit/46158e07f18e29b9b3f5225b0972eacfc35aee4c))

### Performance

- Optimize production build and more compatibility (#33) ([5a0016f](https://github.com/ubugeeei/vize/commit/5a0016f9617e2abd0989bdfeff78344a706bad91))
- Optimize analyzer performance ([b4d58a9](https://github.com/ubugeeei/vize/commit/b4d58a93c4a5f2771c4a3c42848f8207a86e0159))

## [0.0.1-alpha.31] - 2026-01-12

### Added

- Add i18n support to more lint rules and fix editor diagnostics ([453693b](https://github.com/ubugeeei/vize/commit/453693b232857a935df4cb9a41864935b3c11c95))
- Add i18n support to a11y rules and translations ([7748b07](https://github.com/ubugeeei/vize/commit/7748b07798a141fb85ba194a670ae8fad63a7ba0))
- Add a11y lint rules and improve playground UX ([4266ec5](https://github.com/ubugeeei/vize/commit/4266ec5d614ae66e8ceb237e4bceffe72cf830a3))
- Add i18n support for lint messages and fix playground settings ([6f78914](https://github.com/ubugeeei/vize/commit/6f78914104d35233bb33f644a1fe6242c04c45b7))
- Many features... ([18a882f](https://github.com/ubugeeei/vize/commit/18a882f402e5027de55797a8d344fe520e621318))

### Fixed

- Run napi prepublish before napi artifacts ([8232cfc](https://github.com/ubugeeei/vize/commit/8232cfc641844f588f8c1c3129ef7c60d729cb32))

## [0.0.1-alpha.30] - 2026-01-12

### Fixed

- Download only bindings artifacts for native release ([4e5ef2f](https://github.com/ubugeeei/vize/commit/4e5ef2f4f8d53f39696fbeba352f71dd83480b5a))

## [0.0.1-alpha.29] - 2026-01-12

### Fixed

- Use pnpm publish instead of npm publish ([ee0fbd3](https://github.com/ubugeeei/vize/commit/ee0fbd3b3941251259044017fcc9fd72b2653c22))

## [0.0.1-alpha.28] - 2026-01-12

### Added

- Unified docs.rs documentation + fix logo display ([c7c168b](https://github.com/ubugeeei/vize/commit/c7c168b215bd53fb9a1bb17f975971c69a3aca6f))
- Ui improvement ([055ea5d](https://github.com/ubugeeei/vize/commit/055ea5d60f7d8d8d91b46cb38004b2a3049b3172))

### Fixed

- Use --dir instead of --artifacts-dir for napi artifacts ([170b95a](https://github.com/ubugeeei/vize/commit/170b95adac3b846db181882c27482fc724788e4c))

## [0.0.1-alpha.26] - 2026-01-12

### Added

- Many features... ([86023c7](https://github.com/ubugeeei/vize/commit/86023c7e046fb86e31553ca36248e6c9ba223287))

### Fixed

- Add --artifacts-dir flag to napi artifacts command ([d602fea](https://github.com/ubugeeei/vize/commit/d602fea8a94c674b728b4c40d0f09b3c6858c066))

## [0.0.1-alpha.25] - 2026-01-12

### Fixed

- Use working-directory instead of pnpm -C for napi artifacts ([d856fdc](https://github.com/ubugeeei/vize/commit/d856fdc487d782b8096d8baea84ef3d607aadbd1))

## [0.0.1-alpha.24] - 2026-01-12

### Added

- Many features... ([6680b0c](https://github.com/ubugeeei/vize/commit/6680b0cea4fc0ba994ef5d2a2c8323a5aa383abb))

## [0.0.1-alpha.22] - 2026-01-12

### Added

- Use Trusted Publishing (OIDC) for npm packages ([25a6361](https://github.com/ubugeeei/vize/commit/25a6361bce41b96435861522ca42e942b7e2068e))

## [0.0.1-alpha.21] - 2026-01-12

### Fixed

- Skip CLI download in CI environment ([e71dc75](https://github.com/ubugeeei/vize/commit/e71dc75138166166e5735df9b9f0135e10beefab))

## [0.0.1-alpha.14] - 2026-01-12

### Fixed

- Add zig setup for cross-compilation ([efc3dad](https://github.com/ubugeeei/vize/commit/efc3dadcec06ddc10ce35e1d9013b3d7e8941108))

## [0.0.1-alpha.12] - 2026-01-11

### Added

- Many features... ([2c8fc73](https://github.com/ubugeeei/vize/commit/2c8fc7390db69b5bc4138e4b32b449a93317fa46))
- Enable Trusted Publishing for crates.io ([8c461b7](https://github.com/ubugeeei/vize/commit/8c461b74794cfdf0faaaf1bec98dfbb670e304a8))

## [0.0.1-alpha.11] - 2026-01-11

### Fixed

- Use pnpm publish for vite-plugin to resolve workspace:\* dependencies ([72d864e](https://github.com/ubugeeei/vize/commit/72d864e0a2f0dc56e3c5f0ebc37c9d432833b74b))
- Bundle @vizejs/native into vite-plugin and remove unpublished optionalDependencies ([0636016](https://github.com/ubugeeei/vize/commit/0636016d66ed83585a7adec207b415bd74a07753))

## [0.0.1-alpha.10] - 2026-01-11

### Fixed

- Some bugs ([2c62dff](https://github.com/ubugeeei/vize/commit/2c62dff95bb6c7c2a8f6042ce4c72b96de457754))
- Some bugs ([8ca4eb5](https://github.com/ubugeeei/vize/commit/8ca4eb59ae6f9350c389c2780474d15738f97f84))
- Some bugs ([bfdf933](https://github.com/ubugeeei/vize/commit/bfdf9336e5b476291b68efeaf48fffc356360671))

### Performance

- Use u8 vec and byte extends insted String ([441eee9](https://github.com/ubugeeei/vize/commit/441eee9dc2ed3ff9afcc59d46ebd939c2f6adfc8))
