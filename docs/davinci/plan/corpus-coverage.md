<!-- GENERATED FILE — do not edit by hand.
     Regenerate: rust-script tools/commands/davinci/corpus-coverage.rs --write
     Verify:     rust-script tools/commands/davinci/corpus-coverage.rs --check
     Generator:  tools/commands/davinci/corpus-coverage.rs -->

# Corpus construct coverage

Counts of the [taxonomy.toml](./taxonomy.toml) construct dimensions observed in the **hydrated** corpus projects registered in `tests/_fixtures/vue-ecosystem-fixtures.json` (Davinci P0-6). This file is generated; it goes stale whenever the taxonomy, the fixtures manifest, or the set of hydrated fixture submodules changes — regenerate with `--write`, verify with `--check` (byte-compare). The `--check` staleness gate can only join `tests/tooling/davinci-matrices.test.ts` once CI hydrates the full corpus; until then the scope-proof footer below is the honesty mechanism.

## Scan scope

Sources scanned per hydrated project (from the manifest's `vueGlobs`, plus `petiteVueGlobs` for the petite-vue entries):

| project                    | sfc (html) | sfc (pug) | jsx/tsx | html |  js |
| -------------------------- | ---------: | --------: | ------: | ---: | --: |
| `vue-vben-admin`           |        613 |         0 |       0 |    0 |   0 |
| `hoppscotch`               |        365 |         0 |       0 |    0 |   0 |
| `element-plus`             |        823 |         0 |       0 |    0 |   0 |
| `ant-design-vue`           |        731 |         0 |       0 |    0 |   0 |
| `reka-ui`                  |        729 |         0 |       0 |    0 |   0 |
| `primevue`                 |       2610 |         0 |       0 |    0 |   0 |
| `vuetify`                  |       1255 |         0 |       0 |    0 |   0 |
| `naive-ui`                 |       1720 |         0 |       0 |    0 |   0 |
| `voicevox`                 |        134 |         0 |       0 |    0 |   0 |
| `elk`                      |        259 |         0 |       0 |    0 |   0 |
| `misskey`                  |        583 |         0 |       0 |    0 |   0 |
| `directus`                 |        576 |         0 |       0 |    0 |   0 |
| `motion-vue`               |         74 |         0 |       0 |    0 |   0 |
| `shadcn-vue`               |       6514 |         0 |       0 |    0 |   0 |
| `inspira-ui`               |        512 |         0 |       0 |    0 |   0 |
| `vue-charts`               |        193 |         0 |       0 |    0 |   0 |
| `vaul-vue`                 |         21 |         0 |       0 |    0 |   0 |
| `vee-validate`             |         83 |         0 |       0 |    0 |   0 |
| `create-vue`               |         42 |         0 |       0 |    0 |   0 |
| `vue-router`               |        115 |         0 |       0 |    0 |   0 |
| `pinia`                    |         24 |         0 |       0 |    0 |   0 |
| `vue-tui`                  |         19 |         0 |       0 |    0 |   0 |
| `vue-termui`               |         77 |         0 |       0 |    0 |   0 |
| `vue-element-admin`        |        131 |         0 |       0 |    0 |   0 |
| `element`                  |        155 |         0 |       0 |    0 |   0 |
| `lx-music-desktop`         |         92 |        30 |       0 |    0 |   0 |
| `uni-app`                  |         58 |         0 |       0 |    0 |   0 |
| `vue2-elm`                 |         55 |         0 |       0 |    0 |   0 |
| `filebrowser`              |         58 |         0 |       0 |    0 |   0 |
| `docsify`                  |          0 |         0 |       0 |    0 |   0 |
| `dashy`                    |        175 |         0 |       0 |    0 |   0 |
| `vue-devtools-v6`          |        121 |         0 |       0 |    0 |   0 |
| `vant`                     |        129 |         0 |       0 |    0 |   0 |
| `vuepress`                 |         38 |         0 |       0 |    0 |   0 |
| `automa`                   |        215 |         0 |       0 |    0 |   0 |
| `vue-pure-admin`           |        255 |         0 |       0 |    0 |   0 |
| `vue-manage-system`        |         40 |         0 |       0 |    0 |   0 |
| `vitepress`                |         95 |         0 |       0 |    0 |   0 |
| `vux`                      |        310 |         2 |       0 |    0 |   0 |
| `koel`                     |        337 |         0 |       0 |    0 |   0 |
| `better-scroll`            |         65 |         0 |       0 |    0 |   0 |
| `mint-ui`                  |         69 |         0 |       0 |    0 |   0 |
| `scalar`                   |       2007 |         0 |       0 |    0 |   0 |
| `soybean-admin`            |         90 |         0 |       0 |    0 |   0 |
| `zy-player`                |         13 |         0 |       0 |    0 |   0 |
| `bootstrap-vue`            |         22 |         0 |       0 |    0 |   0 |
| `habitica`                 |        352 |         0 |       0 |    0 |   0 |
| `tiny-rdm`                 |        140 |         0 |       0 |    0 |   0 |
| `mealie`                   |        205 |         0 |       0 |    0 |   0 |
| `mall-admin-web`           |         83 |         0 |       0 |    0 |   0 |
| `douyin`                   |        128 |         0 |       0 |    0 |   0 |
| `vuestic-admin`            |         99 |         0 |       0 |    0 |   0 |
| `vue-storefront`           |         45 |         0 |       0 |    0 |   0 |
| `vue-virtual-scroller`     |         18 |         0 |       0 |    0 |   0 |
| `vue-echarts`              |         15 |         0 |       0 |    0 |   0 |
| `gridea`                   |         26 |         0 |       0 |    0 |   0 |
| `dbx`                      |        376 |         0 |       0 |    0 |   0 |
| `vue-material`             |        347 |         0 |       0 |    0 |   0 |
| `datav`                    |         76 |         0 |       0 |    0 |   0 |
| `buefy`                    |        431 |         0 |       0 |    0 |   0 |
| `cube-ui`                  |        164 |         0 |       0 |    0 |   0 |
| `youtube-dl-gui`           |         83 |         0 |       0 |    0 |   0 |
| `solidtime`                |        389 |         0 |       0 |    0 |   0 |
| `crater`                   |        311 |         0 |       0 |    0 |   0 |
| `vue-native-core`          |          0 |         0 |       0 |    0 |   0 |
| `muse-ui`                  |          3 |         0 |       0 |    0 |   0 |
| `inertia`                  |        459 |         0 |       0 |    0 |   0 |
| `gui-for-singbox`          |         98 |         0 |       0 |    0 |   0 |
| `vue-fabric-editor`        |         72 |         0 |       0 |    0 |   0 |
| `vue-grid-layout`          |          5 |         0 |       0 |    0 |   0 |
| `splayer`                  |        154 |         0 |       0 |    0 |   0 |
| `vuelidate`                |         14 |        14 |       0 |    0 |   0 |
| `vuetorrent`               |        145 |         0 |       0 |    0 |   0 |
| `vue-multiselect`          |         36 |         0 |       0 |    0 |   0 |
| `frpc-desktop`             |         13 |         0 |       0 |    0 |   0 |
| `v-charts`                 |         33 |         0 |       0 |    0 |   0 |
| `music-website`            |         47 |         0 |       0 |    0 |   0 |
| `vue-flow`                 |        194 |         0 |       0 |    0 |   0 |
| `mavon-editor`             |          6 |         0 |       0 |    0 |   0 |
| `nutui`                    |       1228 |         0 |       0 |    0 |   0 |
| `nativescript-vue`         |         17 |         0 |       0 |    0 |   0 |
| `sigma-file-manager`       |        298 |         0 |       0 |    0 |   0 |
| `gogocode`                 |        186 |         0 |       0 |    0 |   0 |
| `vue-chartjs`              |         11 |         0 |       0 |    0 |   0 |
| `vuesax`                   |         58 |         0 |       0 |    0 |   0 |
| `cssgridgenerator`         |          9 |         0 |       0 |    0 |   0 |
| `varlet`                   |        226 |         0 |       0 |    0 |   0 |
| `vue-select`               |          4 |         0 |       0 |    0 |   0 |
| `vue-cropper`              |          4 |         0 |       0 |    0 |   0 |
| `vue-draggable-next`       |         30 |         0 |       0 |    0 |   0 |
| `vue-js-modal`             |         19 |         0 |       0 |    0 |   0 |
| `vue-bits`                 |        323 |         0 |       0 |    0 |   0 |
| `vue-netcore`              |        121 |         0 |       0 |    0 |   0 |
| `vue-draggable-plus`       |         36 |         0 |       0 |    0 |   0 |
| `tdesign`                  |         82 |         0 |       0 |    0 |   0 |
| `epic-spinners`            |         50 |         0 |       0 |    0 |   0 |
| `portal-vue`               |         42 |         0 |       0 |    0 |   0 |
| `vuestic-ui`               |       1019 |         0 |       0 |    0 |   0 |
| `piclist`                  |         56 |         0 |       0 |    0 |   0 |
| `vue-draggable-resizable`  |         53 |         0 |       0 |    0 |   0 |
| `pinry`                    |         29 |         0 |       0 |    0 |   0 |
| `vonic`                    |         87 |         0 |       0 |    0 |   0 |
| `laravel-breeze`           |         54 |         0 |       0 |    0 |   0 |
| `frappe-crm`               |        339 |         0 |       0 |    0 |   0 |
| `v-viewer`                 |          6 |         0 |       0 |    0 |   0 |
| `antares`                  |         94 |         0 |       0 |    0 |   0 |
| `heyui`                    |         76 |         0 |       0 |    0 |   0 |
| `vue-data-ui`              |        403 |         0 |       0 |    0 |   0 |
| `splitpanes`               |          2 |         7 |       0 |    0 |   0 |
| `tailwind-config-viewer`   |         29 |         0 |       0 |    0 |   0 |
| `frappe-builder`           |        159 |         0 |       0 |    0 |   0 |
| `vue-uploader`             |          8 |         0 |       0 |    0 |   0 |
| `vant-demo`                |         15 |         0 |       0 |    0 |   0 |
| `vue-dropzone`             |         18 |         0 |       0 |    0 |   0 |
| `multiple-select`          |         62 |         0 |       0 |    0 |   0 |
| `alexandrie`               |        202 |         0 |       0 |    0 |   0 |
| `vue-fullpage-js`          |          2 |         0 |       0 |    0 |   0 |
| `arco-design-pro-vue`      |         86 |         0 |       0 |    0 |   0 |
| `vue-datepicker`           |         19 |         0 |       0 |    0 |   0 |
| `layoutit-grid`            |        112 |         0 |       0 |    0 |   0 |
| `jellyfin-vue`             |        136 |         0 |       0 |    0 |   0 |
| `lew-ui`                   |        429 |         0 |       0 |    0 |   0 |
| `vue-sonner`               |         26 |         0 |       0 |    0 |   0 |
| `element-plus-x`           |        334 |         0 |       0 |    0 |   0 |
| `vue-vine`                 |          2 |         0 |       0 |    0 |   0 |
| `vue-calendar`             |          2 |         0 |       0 |    0 |   0 |
| `prevue`                   |         24 |         0 |       0 |    0 |   0 |
| `bym-vue-echarts`          |         37 |         0 |       0 |    0 |   0 |
| `vue-apexcharts`           |         11 |         0 |       0 |    0 |   0 |
| `vue-lottie`               |          2 |         0 |       0 |    0 |   0 |
| `vue-cal-v4`               |          0 |        16 |       0 |    0 |   0 |
| `airi`                     |        586 |         0 |       0 |    0 |   0 |
| `vuefes-japan-speakers`    |         15 |         0 |       0 |    0 |   0 |
| `mobile-web-best-practice` |          7 |         0 |       0 |    0 |   0 |
| `wave-ui`                  |          0 |       219 |       0 |    0 |   0 |
| `dho-web-client`           |          4 |       211 |       0 |    0 |   0 |
| `vue3-admin-design`        |          7 |         0 |       0 |    0 |   0 |
| `vue3-antd-admin`          |         99 |         0 |       0 |    0 |   0 |
| `vue-core-vapor`           |        105 |         0 |       0 |    0 |   0 |
| `vue-jsx-vapor`            |          1 |         0 |       0 |    0 |   0 |
| `wakapi`                   |          0 |         0 |       0 |   29 |   6 |
| `petite-vue`               |          0 |         0 |       0 |    6 |   0 |

## Per-construct counts (hydrated projects only)

### Dimension 1: element_kind (start-tag classes)

| project                    | native | component |  slot | template |   svg | mathml |
| -------------------------- | -----: | --------: | ----: | -------: | ----: | -----: |
| `vue-vben-admin`           |   1002 |      2093 |   368 |      369 |  1362 |      0 |
| `hoppscotch`               |   3718 |      2272 |    14 |      444 |    47 |      0 |
| `element-plus`             |   2140 |      3534 |   284 |      460 |   109 |      0 |
| `ant-design-vue`           |   1666 |      4435 |     9 |      636 |    19 |      0 |
| `reka-ui`                  |    848 |      4540 |   466 |       83 |    14 |      0 |
| `primevue`                 |  17837 |      8840 |   472 |     1090 |   699 |      0 |
| `vuetify`                  |   2665 |      8672 |    34 |      971 |    18 |      0 |
| `naive-ui`                 |   2012 |      9113 |     5 |     1030 |    51 |      0 |
| `voicevox`                 |    550 |       867 |    24 |       73 |    36 |      0 |
| `elk`                      |   1494 |       850 |    69 |      248 |    30 |      0 |
| `misskey`                  |   6106 |      4541 |   119 |     2281 |   171 |      0 |
| `directus`                 |   2313 |      4585 |   231 |      898 |   154 |      0 |
| `motion-vue`               |    430 |       147 |     7 |        0 |    12 |      0 |
| `shadcn-vue`               |   9116 |     31458 |  4283 |      370 |   323 |      0 |
| `inspira-ui`               |   1368 |      1025 |   102 |      331 |   245 |      0 |
| `vue-charts`               |    425 |      1732 |    22 |      162 |    68 |      0 |
| `vaul-vue`                 |    210 |        92 |     5 |        0 |    37 |      0 |
| `vee-validate`             |    293 |        48 |     6 |        4 |    37 |      0 |
| `create-vue`               |    144 |        54 |    12 |       40 |    40 |      0 |
| `vue-router`               |    564 |       167 |     2 |        8 |     0 |      0 |
| `pinia`                    |    174 |        13 |     2 |       13 |    28 |      0 |
| `vue-tui`                  |      0 |        98 |     6 |        1 |     0 |      0 |
| `vue-termui`               |     33 |       631 |     1 |       12 |     0 |      0 |
| `vue-element-admin`        |    834 |       551 |     7 |       79 |    10 |      0 |
| `element`                  |    956 |       380 |   110 |       37 |    43 |      0 |
| `lx-music-desktop`         |   1402 |       422 |    11 |       15 |   376 |      0 |
| `uni-app`                  |    116 |        27 |    15 |        3 |   195 |      0 |
| `vue2-elm`                 |    358 |        46 |     6 |        0 |   306 |      0 |
| `filebrowser`              |    947 |        47 |     6 |       26 |     0 |      0 |
| `docsify`                  |      0 |         0 |     0 |        0 |     0 |      0 |
| `dashy`                    |   1718 |       316 |     9 |       34 |    22 |      0 |
| `vue-devtools-v6`          |    595 |       366 |    30 |       71 |     2 |      0 |
| `vant`                     |    209 |      1610 |    10 |      100 |     5 |      0 |
| `vuepress`                 |     89 |        42 |    12 |        6 |     9 |      0 |
| `automa`                   |   1788 |      1681 |    62 |      239 |    11 |      0 |
| `vue-pure-admin`           |   1017 |      1946 |     8 |      237 |    31 |      0 |
| `vue-manage-system`        |    303 |       540 |     4 |       75 |     0 |      0 |
| `vitepress`                |    307 |       109 |   129 |       95 |    61 |      0 |
| `vux`                      |   2604 |       912 |   123 |       21 |    35 |      0 |
| `koel`                     |   1360 |      1351 |    82 |      365 |    28 |      0 |
| `better-scroll`            |    600 |        14 |     0 |        0 |     0 |      0 |
| `mint-ui`                  |    340 |       236 |    36 |        0 |     0 |      0 |
| `scalar`                   |   2058 |      1735 |  1837 |      589 | 21278 |      0 |
| `soybean-admin`            |    175 |       377 |    33 |       43 |    64 |      0 |
| `zy-player`                |    463 |       283 |     0 |       27 |    91 |      0 |
| `bootstrap-vue`            |    285 |       253 |     0 |       42 |    43 |      0 |
| `habitica`                 |   7106 |       888 |    47 |       93 |     4 |      0 |
| `tiny-rdm`                 |    197 |       828 |    14 |      139 |   337 |      0 |
| `mealie`                   |    799 |      2435 |    58 |      385 |     2 |      0 |
| `mall-admin-web`           |    479 |      1645 |     1 |      297 |     6 |      0 |
| `douyin`                   |   5333 |       610 |    23 |      157 |     3 |      0 |
| `vuestic-admin`            |    471 |       402 |     5 |       54 |   126 |      0 |
| `vue-storefront`           |    232 |       450 |     0 |       51 |     0 |      0 |
| `vue-virtual-scroller`     |    139 |        36 |    14 |       23 |     0 |      0 |
| `vue-echarts`              |    122 |        41 |     4 |       16 |     0 |      0 |
| `gridea`                   |    249 |       310 |     1 |        8 |     0 |      0 |
| `dbx`                      |  12140 |      8013 |   112 |      493 |    37 |      0 |
| `vue-material`             |   2398 |      2507 |   107 |       18 |    41 |      0 |
| `datav`                    |    180 |         2 |    38 |       10 |   668 |      0 |
| `buefy`                    |   3608 |      2644 |   108 |      184 |    22 |      0 |
| `cube-ui`                  |    852 |       522 |    80 |       47 |     0 |      0 |
| `youtube-dl-gui`           |    836 |       235 |    22 |       64 |     8 |      0 |
| `solidtime`                |   1479 |      2008 |   188 |      272 |    99 |      0 |
| `crater`                   |   1659 |      2684 |    79 |      391 |   284 |      0 |
| `vue-native-core`          |      0 |         0 |     0 |        0 |     0 |      0 |
| `muse-ui`                  |      7 |        24 |     0 |        1 |     0 |      0 |
| `inertia`                  |   3448 |       589 |    20 |      136 |     7 |      0 |
| `gui-for-singbox`          |    661 |       731 |    33 |      160 |     7 |      0 |
| `vue-fabric-editor`        |    505 |       435 |     4 |       41 |     8 |      0 |
| `vue-grid-layout`          |     39 |         4 |     2 |        0 |     0 |      0 |
| `splayer`                  |    697 |      1946 |     8 |      237 |     4 |      0 |
| `vuelidate`                |    345 |        25 |     0 |        1 |     0 |      0 |
| `vuetorrent`               |    421 |      2136 |    17 |      238 |     0 |      0 |
| `vue-multiselect`          |   1002 |        31 |    15 |       13 |     0 |      0 |
| `frpc-desktop`             |    260 |       454 |     2 |      155 |     0 |      0 |
| `v-charts`                 |     66 |        55 |     0 |        0 |     2 |      0 |
| `music-website`            |    257 |       438 |     2 |       25 |     4 |      0 |
| `vue-flow`                 |    425 |       420 |    27 |       68 |   154 |      0 |
| `mavon-editor`             |    146 |         8 |     8 |        0 |     0 |      0 |
| `nutui`                    |   2096 |      4257 |   409 |      441 |   893 |      0 |
| `nativescript-vue`         |      1 |       112 |     0 |        9 |     0 |      0 |
| `sigma-file-manager`       |   1736 |      1960 |   122 |      146 |     5 |      0 |
| `gogocode`                 |    711 |       290 |    14 |       29 |     2 |      0 |
| `vue-chartjs`              |      0 |        11 |     0 |        0 |     0 |      0 |
| `vuesax`                   |    311 |        50 |    63 |        2 |     2 |      0 |
| `cssgridgenerator`         |    117 |         9 |     2 |        4 |    10 |      0 |
| `varlet`                   |    828 |      2768 |   196 |      206 |    48 |      0 |
| `vue-select`               |     18 |         1 |    11 |        0 |     4 |      0 |
| `vue-cropper`              |    195 |         0 |     1 |        1 |     0 |      0 |
| `vue-draggable-next`       |    255 |         7 |     1 |       36 |     0 |      0 |
| `vue-js-modal`             |    103 |        13 |     2 |        1 |     4 |      0 |
| `vue-bits`                 |   1442 |      1876 |    34 |      582 |   283 |      0 |
| `vue-netcore`              |    996 |       550 |    33 |      143 |     0 |      0 |
| `vue-draggable-plus`       |    175 |        69 |     0 |        0 |     0 |      0 |
| `tdesign`                  |   2102 |       350 |    10 |       75 |   320 |      0 |
| `epic-spinners`            |    611 |        87 |     1 |        0 |     0 |      0 |
| `portal-vue`               |    272 |        42 |     3 |        6 |     0 |      0 |
| `vuestic-ui`               |   3106 |      5142 |   289 |      801 |   454 |      0 |
| `piclist`                  |   1292 |       772 |    29 |      105 |    19 |      0 |
| `vue-draggable-resizable`  |    116 |       156 |     2 |       40 |     0 |      0 |
| `pinry`                    |    241 |        95 |     0 |       10 |     0 |      0 |
| `vonic`                    |    724 |        95 |    21 |        0 |    14 |      0 |
| `laravel-breeze`           |    336 |       220 |    26 |       10 |    58 |      0 |
| `frappe-crm`               |   2727 |      1945 |    45 |      437 |   366 |      0 |
| `v-viewer`                 |    144 |         5 |     1 |        4 |     0 |      0 |
| `antares`                  |   2794 |       739 |     7 |       76 |     1 |      0 |
| `heyui`                    |    574 |        55 |    79 |       47 |     5 |      0 |
| `vue-data-ui`              |   3864 |      2753 |  1469 |     4440 |  2481 |      0 |
| `splitpanes`               |    482 |       121 |     2 |       24 |     6 |      0 |
| `tailwind-config-viewer`   |    111 |        51 |     5 |        1 |     4 |      0 |
| `frappe-builder`           |   1138 |       563 |    26 |      146 |    29 |      0 |
| `vue-uploader`             |     33 |         7 |     7 |        0 |     0 |      0 |
| `vant-demo`                |     29 |       145 |     1 |       10 |     0 |      0 |
| `vue-dropzone`             |    252 |        26 |     1 |        0 |     0 |      0 |
| `multiple-select`          |   1704 |       106 |     2 |        0 |     0 |      0 |
| `alexandrie`               |   3136 |       794 |    17 |       45 |   138 |      0 |
| `vue-fullpage-js`          |     32 |         1 |     1 |        0 |     2 |      0 |
| `arco-design-pro-vue`      |    224 |       823 |     2 |      123 |     0 |      0 |
| `vue-datepicker`           |    103 |        39 |    71 |       49 |     0 |      0 |
| `layoutit-grid`            |    316 |       163 |    13 |       26 |   115 |      0 |
| `jellyfin-vue`             |    307 |      1300 |    48 |      156 |     4 |      0 |
| `lew-ui`                   |   1155 |      1506 |    53 |      150 |     5 |      0 |
| `vue-sonner`               |    148 |        48 |    12 |       23 |    16 |      0 |
| `element-plus-x`           |   2159 |      1250 |    72 |      273 |   238 |      0 |
| `vue-vine`                 |      2 |         2 |     0 |        0 |     0 |      0 |
| `vue-calendar`             |     48 |         0 |     0 |        0 |     8 |      0 |
| `prevue`                   |    161 |        92 |     0 |        5 |     0 |      0 |
| `bym-vue-echarts`          |    343 |        65 |     0 |        2 |     5 |      0 |
| `vue-apexcharts`           |     30 |        11 |     0 |        0 |     0 |      0 |
| `vue-lottie`               |      9 |         0 |     0 |        0 |     0 |      0 |
| `vue-cal-v4`               |   1424 |       470 |    33 |       58 |     2 |      0 |
| `airi`                     |   5660 |      2548 |   169 |      463 |    67 |      0 |
| `vuefes-japan-speakers`    |    177 |        20 |     0 |       11 |     3 |      0 |
| `mobile-web-best-practice` |     47 |        29 |     0 |        0 |     0 |      0 |
| `wave-ui`                  |   5549 |      2828 |   155 |     1558 |    39 |      0 |
| `dho-web-client`           |   3251 |      1665 |    30 |      218 |     4 |      0 |
| `vue3-admin-design`        |      1 |        91 |     0 |        1 |     0 |      0 |
| `vue3-antd-admin`          |    212 |       529 |    34 |      137 |     2 |      0 |
| `vue-core-vapor`           |    507 |        89 |     9 |        5 |     2 |      0 |
| `vue-jsx-vapor`            |      0 |         0 |     0 |        0 |     0 |      0 |
| `wakapi`                   |   1418 |         0 |     0 |        2 |     2 |      0 |
| `petite-vue`               |     89 |         0 |     0 |        2 |     5 |      0 |
| **total sites**            | 178334 |    176418 | 13745 |    26544 | 33598 |      0 |
| **projects seen**          |    137 |       134 |   117 |      117 |    97 |      0 |

### Dimension 2: directive (attribute names, incl. `:` / `@` shorthand)

| project                    |  v-if | v-else-if | v-else | v-for |  v-on | v-bind | v-model | v-show | v-html | v-text | v-once | v-memo | v-cloak | v-pre | custom |
| -------------------------- | ----: | --------: | -----: | ----: | ----: | -----: | ------: | -----: | -----: | -----: | -----: | -----: | ------: | ----: | -----: |
| `vue-vben-admin`           |   391 |        32 |     49 |   104 |   631 |   2001 |     270 |     23 |      3 |      0 |      0 |      0 |       0 |     0 |     24 |
| `hoppscotch`               |   935 |       123 |    173 |   208 |  1884 |   4652 |     371 |      7 |      3 |      1 |      0 |      0 |       0 |     0 |    399 |
| `element-plus`             |   361 |        19 |     60 |   187 |   993 |   4138 |     641 |     58 |      4 |      5 |      0 |      0 |       0 |     0 |     25 |
| `ant-design-vue`           |   132 |        22 |     64 |    43 |   506 |   1928 |     619 |      1 |      5 |      0 |      0 |      0 |       0 |     0 |      4 |
| `reka-ui`                  |   176 |         8 |     68 |   305 |   597 |   3828 |     214 |      4 |      0 |      0 |      0 |      3 |       0 |     0 |      0 |
| `primevue`                 |   777 |        82 |    140 |   281 |  1677 |  12898 |     929 |     17 |      8 |      0 |      0 |      0 |       0 |     0 |    233 |
| `vuetify`                  |   337 |        11 |     51 |   376 |   601 |   3277 |    1139 |      8 |     13 |     64 |      0 |      0 |       0 |     0 |     51 |
| `naive-ui`                 |    86 |         0 |     17 |    72 |   986 |   4065 |    1266 |      3 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `voicevox`                 |   181 |        10 |     30 |    76 |   433 |    990 |     105 |      4 |      4 |      0 |      0 |      0 |       0 |     0 |     15 |
| `elk`                      |   410 |        57 |    100 |    71 |   299 |   1451 |      74 |      6 |      1 |      0 |      0 |      0 |       0 |     0 |      2 |
| `misskey`                  |  1602 |       549 |    325 |   294 |  1534 |   7325 |     898 |     47 |     13 |     17 |      3 |      0 |       0 |     0 |    204 |
| `directus`                 |  1355 |       163 |    332 |   215 |  1973 |   5319 |     634 |     19 |      3 |      0 |      0 |      0 |       0 |     0 |    312 |
| `motion-vue`               |    20 |         0 |      1 |    21 |    93 |    441 |       7 |     11 |      0 |      0 |      0 |      0 |       0 |     0 |     18 |
| `shadcn-vue`               |   767 |        88 |    182 |   916 |   957 |  16307 |     384 |      1 |     13 |      0 |      0 |      0 |       0 |     0 |      2 |
| `inspira-ui`               |   124 |        12 |     14 |   101 |   187 |   2075 |      13 |     10 |      0 |      2 |      0 |      0 |       0 |     0 |      0 |
| `vue-charts`               |    31 |         2 |      5 |    28 |     8 |   1702 |       3 |      2 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vaul-vue`                 |     0 |         0 |      0 |     0 |    20 |     24 |       8 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vee-validate`             |    20 |         0 |      9 |     8 |    51 |    123 |      68 |      2 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `create-vue`               |     0 |         0 |      0 |     0 |     4 |      0 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-router`               |    13 |         0 |      6 |     1 |    17 |     49 |       6 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `pinia`                    |     8 |         6 |      1 |     3 |    49 |     32 |       4 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-tui`                  |    12 |         5 |      0 |     6 |    14 |     69 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-termui`               |    14 |         2 |      1 |    32 |    69 |    861 |      12 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-element-admin`        |    43 |         0 |      9 |    39 |   186 |    510 |      82 |     28 |      3 |      1 |      0 |      0 |       0 |     0 |     25 |
| `element`                  |   221 |         1 |     30 |    59 |   437 |    931 |      59 |     70 |      6 |      6 |      0 |      0 |       0 |     0 |     26 |
| `lx-music-desktop`         |   140 |        18 |     43 |    53 |   586 |   1667 |      65 |     27 |      0 |     16 |     13 |      0 |       0 |     0 |      0 |
| `uni-app`                  |    20 |         3 |      5 |    13 |   110 |     95 |       1 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue2-elm`                 |    49 |         0 |      8 |    18 |    41 |     87 |      13 |     13 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `filebrowser`              |   148 |        19 |     27 |    24 |   197 |    394 |      66 |     10 |      1 |      0 |      0 |      0 |       0 |     0 |      3 |
| `docsify`                  |     0 |         0 |      0 |     0 |     0 |      0 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `dashy`                    |   381 |        23 |     46 |   120 |   241 |    851 |      43 |      4 |     20 |      0 |      0 |      0 |       0 |     0 |    111 |
| `vue-devtools-v6`          |   143 |         7 |     21 |    32 |   227 |    324 |      55 |      2 |      7 |      0 |      0 |      0 |       0 |     0 |     51 |
| `vant`                     |    28 |         2 |      3 |    48 |   320 |   1457 |     297 |      3 |      4 |      0 |      0 |      0 |       0 |     0 |      3 |
| `vuepress`                 |    36 |         1 |      9 |     7 |    32 |     72 |       0 |      1 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `automa`                   |   470 |        21 |    110 |   222 |  1033 |   1839 |     260 |     14 |      0 |      3 |      0 |      0 |       0 |     0 |    126 |
| `vue-pure-admin`           |   138 |         5 |     26 |    72 |   516 |   1855 |     270 |     41 |      5 |      1 |      0 |      0 |       0 |     0 |    107 |
| `vue-manage-system`        |    26 |        10 |     14 |    20 |    70 |    358 |      70 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      4 |
| `vitepress`                |    93 |        12 |     11 |    21 |    43 |    287 |       1 |      2 |     27 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vux`                      |   122 |         0 |      7 |   106 |   537 |   1492 |     371 |     71 |     54 |      2 |      0 |      0 |       0 |     0 |     49 |
| `koel`                     |   615 |        31 |    141 |   103 |   727 |   1156 |     136 |     46 |      8 |      0 |      0 |      0 |       0 |     0 |     59 |
| `better-scroll`            |     4 |         0 |      3 |    62 |    95 |     89 |       0 |     34 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `mint-ui`                  |    31 |         0 |      1 |    33 |   104 |    191 |      39 |     20 |      3 |     12 |      0 |      0 |       0 |     0 |      4 |
| `scalar`                   |  2384 |      7649 |    182 |   150 |   828 |   4973 |      95 |     19 |      5 |      1 |      0 |      0 |       0 |     1 |      0 |
| `soybean-admin`            |    53 |         4 |      8 |    14 |    87 |    412 |      59 |      8 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `zy-player`                |    83 |         0 |      2 |    25 |   279 |    230 |      72 |     51 |      1 |      0 |      0 |      0 |       0 |     0 |      7 |
| `bootstrap-vue`            |    72 |         1 |      4 |    13 |    13 |    110 |       4 |      0 |      6 |      0 |      0 |      0 |       0 |     0 |      3 |
| `habitica`                 |   952 |        28 |     98 |   204 |  1128 |   2551 |     220 |     18 |    640 |      0 |    558 |      0 |       0 |     0 |     80 |
| `tiny-rdm`                 |    81 |         4 |     22 |    15 |   308 |   1658 |     164 |     20 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `mealie`                   |   490 |        35 |     57 |   117 |   633 |   2222 |     446 |      5 |      7 |      0 |      0 |      4 |       0 |     0 |      3 |
| `mall-admin-web`           |    24 |         1 |      6 |    56 |   356 |    759 |     311 |     25 |      0 |      0 |      0 |      0 |       0 |     0 |     24 |
| `douyin`                   |   376 |         8 |     75 |    95 |   736 |    525 |     154 |     38 |      2 |      3 |      0 |      0 |       0 |     0 |     42 |
| `vuestic-admin`            |    55 |         1 |     15 |    26 |   108 |    292 |      70 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-storefront`           |    37 |         2 |     20 |    48 |   105 |    444 |      35 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |     58 |
| `vue-virtual-scroller`     |    17 |         0 |      2 |     2 |    25 |    144 |      15 |      1 |      0 |      0 |      0 |      0 |       0 |     0 |      2 |
| `vue-echarts`              |     5 |         0 |      2 |     6 |    22 |     93 |      13 |      0 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `gridea`                   |    69 |         2 |      7 |    26 |    90 |    375 |      85 |      2 |      1 |      0 |      0 |      0 |       0 |     0 |      1 |
| `dbx`                      |  2901 |       573 |    661 |   703 |  4139 |   8420 |    1298 |     25 |     42 |      0 |      0 |      0 |       0 |     0 |      2 |
| `vue-material`             |   101 |        11 |     23 |    24 |   193 |    862 |     166 |      6 |      8 |      0 |     15 |      0 |       0 |     1 |      0 |
| `datav`                    |    61 |         0 |      2 |    48 |    10 |   1390 |       0 |      0 |      6 |      0 |      0 |      0 |       0 |     0 |      0 |
| `buefy`                    |   270 |        14 |     62 |    83 |   532 |   2580 |     386 |     33 |      9 |      0 |      0 |      0 |       0 |     0 |      9 |
| `cube-ui`                  |    71 |         1 |     12 |    81 |   347 |    657 |      76 |     37 |     19 |      0 |      0 |      0 |       0 |     0 |      2 |
| `youtube-dl-gui`           |   130 |        18 |     51 |    39 |   111 |    622 |     129 |      2 |      3 |      0 |      0 |      0 |       0 |     0 |      0 |
| `solidtime`                |   461 |        35 |     92 |    78 |   654 |   1818 |     302 |      4 |      5 |      0 |      0 |      0 |       0 |     0 |      0 |
| `crater`                   |   547 |        12 |    131 |    73 |   761 |   3318 |     499 |     41 |      9 |      0 |      0 |      0 |       0 |     0 |      3 |
| `vue-native-core`          |     0 |         0 |      0 |     0 |     0 |      0 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `muse-ui`                  |     1 |         0 |      0 |     0 |     7 |      8 |       4 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `inertia`                  |   305 |         0 |     20 |    81 |   670 |    413 |     133 |      0 |      1 |      1 |      0 |      0 |       0 |     0 |      0 |
| `gui-for-singbox`          |   189 |        27 |     42 |    70 |   292 |    573 |     274 |     17 |      2 |      0 |      0 |      1 |       0 |     0 |     75 |
| `vue-fabric-editor`        |    79 |         3 |     11 |    36 |   241 |    358 |     103 |     15 |      7 |      0 |      0 |      0 |       0 |     0 |      3 |
| `vue-grid-layout`          |     1 |         0 |      0 |     2 |    21 |     39 |      11 |      1 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `splayer`                  |   389 |        40 |    108 |    96 |   495 |   1611 |     114 |     10 |      8 |      0 |      0 |      0 |       0 |     0 |     14 |
| `vuelidate`                |    51 |         2 |      1 |    19 |    39 |     78 |      56 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vuetorrent`               |   205 |        32 |     65 |    45 |   418 |   1334 |     457 |      5 |      3 |      0 |      0 |      0 |       0 |     0 |      5 |
| `vue-multiselect`          |    12 |         0 |      0 |     2 |    42 |    149 |       9 |      6 |      2 |      2 |      0 |      0 |       0 |     0 |      0 |
| `frpc-desktop`             |    54 |         6 |     13 |    10 |    71 |    303 |      71 |      3 |     15 |      0 |      0 |      0 |       0 |     0 |      6 |
| `v-charts`                 |     0 |         0 |      0 |     3 |    14 |    129 |       0 |      0 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `music-website`            |    38 |         3 |     19 |    20 |   167 |    327 |      80 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-flow`                 |    71 |        25 |     11 |    15 |   183 |    733 |      65 |      0 |      0 |      0 |      0 |      1 |       0 |     0 |      0 |
| `mavon-editor`             |    45 |         1 |      0 |     3 |    79 |    113 |       9 |     15 |      3 |      0 |      0 |      0 |       0 |     0 |      0 |
| `nutui`                    |   541 |        14 |     94 |   190 |  1431 |   2954 |     673 |     41 |     26 |      0 |      0 |      0 |       0 |     0 |      0 |
| `nativescript-vue`         |     2 |         0 |      0 |     2 |    30 |     30 |       1 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `sigma-file-manager`       |   543 |       105 |    136 |   119 |   752 |   2136 |     107 |      4 |      7 |      0 |      0 |      1 |       0 |     0 |      1 |
| `gogocode`                 |    34 |         4 |      9 |    31 |   149 |    167 |      40 |      4 |      5 |      0 |      0 |      0 |       0 |     0 |      3 |
| `vue-chartjs`              |     0 |         0 |      0 |     0 |     1 |     22 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vuesax`                   |    97 |         4 |      9 |     8 |   108 |    321 |       4 |      9 |      3 |      6 |      0 |      0 |       0 |     0 |      2 |
| `cssgridgenerator`         |     9 |         0 |      4 |     6 |    20 |     19 |       2 |      0 |      3 |      0 |      0 |      0 |       0 |     0 |      0 |
| `varlet`                   |   226 |        16 |     27 |    73 |   574 |   2764 |     511 |     12 |      1 |      0 |      0 |      0 |       0 |     0 |     61 |
| `vue-select`               |     4 |         0 |      1 |     2 |     8 |     39 |       1 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `vue-cropper`              |     7 |         0 |      1 |     0 |    83 |     38 |      23 |      5 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-draggable-next`       |     0 |         0 |      0 |     7 |    42 |    101 |      10 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-js-modal`             |    16 |         0 |      2 |     4 |    50 |     61 |       0 |      0 |      3 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-bits`                 |   120 |        27 |     31 |    83 |   252 |   5371 |     989 |      1 |      3 |      0 |      0 |      0 |       0 |     0 |      2 |
| `vue-netcore`              |   112 |        65 |     30 |    94 |   357 |   1900 |     123 |     62 |      9 |      2 |      0 |      0 |       0 |     0 |      2 |
| `vue-draggable-plus`       |     0 |         0 |      0 |    46 |    31 |    102 |      26 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |     13 |
| `tdesign`                  |    77 |        10 |      6 |   116 |   155 |    682 |      28 |     30 |      4 |      0 |      0 |      0 |       0 |     0 |      0 |
| `epic-spinners`            |     8 |         0 |      0 |    12 |     9 |    250 |       4 |      4 |      0 |      4 |      0 |      0 |       0 |     0 |      4 |
| `portal-vue`               |    11 |         0 |      4 |     6 |    25 |     51 |       2 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vuestic-ui`               |   338 |        25 |     47 |   344 |   742 |   3599 |    1346 |     20 |      8 |      5 |      0 |      0 |       0 |     0 |      0 |
| `piclist`                  |   345 |        32 |     46 |    65 |   457 |   1430 |     229 |     10 |      5 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-draggable-resizable`  |     5 |         0 |      0 |     4 |    19 |    189 |      49 |      1 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `pinry`                    |    20 |         0 |      0 |     6 |    51 |    129 |      21 |     19 |      3 |      0 |      0 |      0 |       0 |     0 |      4 |
| `vonic`                    |    42 |         0 |      0 |    18 |   137 |    147 |      21 |      2 |     13 |     33 |      0 |      0 |       0 |     0 |     42 |
| `laravel-breeze`           |    26 |         0 |      4 |     0 |    38 |    130 |      40 |     12 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `frappe-crm`               |   907 |       244 |    154 |   126 |  1084 |   2980 |     460 |     21 |     20 |      0 |      0 |      0 |       0 |     0 |      0 |
| `v-viewer`                 |     1 |         0 |      1 |     3 |    35 |     21 |       6 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `antares`                  |   416 |        45 |     44 |    57 |   793 |   1631 |     275 |     46 |     18 |      0 |      1 |      0 |       0 |     0 |     12 |
| `heyui`                    |   200 |        17 |     50 |    55 |   211 |    520 |      28 |     13 |      2 |      0 |      0 |      0 |       0 |     0 |      1 |
| `vue-data-ui`              |  2712 |        29 |    170 |   696 |  2542 |  16404 |     143 |     12 |    111 |      1 |      0 |      0 |       0 |     0 |      5 |
| `splitpanes`               |     6 |         0 |      0 |     7 |    24 |     30 |       1 |      1 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `tailwind-config-viewer`   |     6 |         0 |      0 |    24 |    15 |    138 |       6 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `frappe-builder`           |   281 |        29 |     47 |    77 |   611 |   1275 |      68 |     65 |     11 |      1 |      0 |      0 |       0 |     0 |      4 |
| `vue-uploader`             |     0 |         0 |      0 |     2 |     6 |     40 |       0 |      5 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vant-demo`                |     0 |         0 |      0 |     6 |    31 |     34 |       2 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-dropzone`             |     3 |         0 |      0 |     5 |    27 |     75 |       1 |      0 |     28 |      0 |      0 |      0 |       0 |     2 |      0 |
| `multiple-select`          |     2 |         0 |      1 |     4 |    39 |     81 |      17 |      0 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `alexandrie`               |   311 |        31 |     61 |    97 |   422 |    828 |     152 |      8 |     10 |     11 |      0 |      0 |       0 |     0 |      0 |
| `vue-fullpage-js`          |     0 |         0 |      0 |     0 |     4 |      1 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `arco-design-pro-vue`      |    49 |         9 |     19 |    23 |    53 |    728 |      29 |      2 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `vue-datepicker`           |    90 |         0 |      7 |    27 |   126 |    398 |       1 |      4 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `layoutit-grid`            |    71 |         0 |      1 |    26 |   161 |    363 |      12 |      9 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `jellyfin-vue`             |   217 |        18 |     33 |    82 |   212 |   1019 |     123 |      4 |      0 |      0 |      0 |      0 |       0 |     0 |      2 |
| `lew-ui`                   |   218 |        19 |     53 |    81 |   443 |   1499 |     294 |     22 |      0 |      0 |      0 |      0 |       0 |     0 |     39 |
| `vue-sonner`               |    22 |         4 |     14 |     7 |    63 |    134 |       3 |      0 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `element-plus-x`           |   314 |        39 |     79 |    44 |   551 |   1408 |     129 |      8 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-vine`                 |     0 |         0 |      0 |     0 |     0 |      0 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-calendar`             |     5 |         0 |      0 |     5 |    19 |     35 |       2 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `prevue`                   |     6 |         0 |      2 |     3 |    50 |     71 |      10 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `bym-vue-echarts`          |     2 |         0 |      1 |     1 |     9 |     66 |       2 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-apexcharts`           |     0 |         0 |      0 |     0 |     4 |     20 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-lottie`               |     0 |         0 |      0 |     0 |     5 |      4 |       1 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-cal-v4`               |    60 |        10 |      3 |    15 |   112 |    382 |      11 |      1 |     18 |      0 |      0 |      0 |       0 |     0 |      6 |
| `airi`                     |   818 |       102 |    192 |   215 |   924 |   6391 |     739 |      5 |      6 |      0 |      0 |      0 |       0 |     0 |     77 |
| `vuefes-japan-speakers`    |    25 |         0 |      9 |    14 |    25 |     96 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `mobile-web-best-practice` |     5 |         0 |      0 |     3 |    26 |     24 |      10 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      3 |
| `wave-ui`                  |   234 |        52 |     44 |    56 |   396 |   1904 |     176 |     24 |     51 |      1 |      0 |      0 |       0 |     0 |      6 |
| `dho-web-client`           |   907 |        60 |    125 |   125 |   727 |   3219 |     183 |     81 |      5 |      0 |      0 |      0 |       0 |     0 |     24 |
| `vue3-admin-design`        |     0 |         0 |      0 |     0 |    25 |     40 |      12 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue3-antd-admin`          |    60 |         2 |     12 |    39 |   162 |    530 |      47 |      5 |      1 |      0 |      0 |      0 |       0 |     0 |      0 |
| `vue-core-vapor`           |    54 |         4 |      7 |    26 |   206 |     86 |       5 |     14 |      0 |      0 |      0 |      0 |       0 |     0 |      1 |
| `vue-jsx-vapor`            |     0 |         0 |      0 |     0 |     0 |      0 |       0 |      0 |      0 |      0 |      0 |      0 |       0 |     0 |      0 |
| `wakapi`                   |     6 |         0 |      0 |     2 |    47 |     55 |       9 |     10 |      1 |      0 |      0 |      0 |      15 |     0 |      9 |
| `petite-vue`               |     3 |         0 |      1 |     9 |    15 |     17 |       8 |      4 |      1 |      0 |      0 |      0 |       1 |     0 |      9 |
| **total sites**            | 31734 |     10898 |   5547 |  9926 | 49015 | 193012 |   22693 |   1626 |   1395 |    201 |    590 |     10 |      16 |     4 |   2534 |
| **projects seen**          |   126 |        84 |    110 |   128 |   138 |    137 |     126 |    101 |     80 |     25 |      5 |      5 |       2 |     3 |     72 |

### Dimension 3: modifier_class (modifier tokens on the applicable directive)

| project                    | event |  key | mouse-button | v-bind | v-model |
| -------------------------- | ----: | ---: | -----------: | -----: | ------: |
| `vue-vben-admin`           |    34 |    5 |            0 |      2 |       0 |
| `hoppscotch`               |    72 |   23 |            2 |      0 |       1 |
| `element-plus`             |   112 |   33 |            0 |      0 |       2 |
| `ant-design-vue`           |    11 |    1 |            0 |      0 |       1 |
| `reka-ui`                  |    63 |  102 |            3 |      0 |       0 |
| `primevue`                 |    18 |   34 |            0 |      0 |       3 |
| `vuetify`                  |    53 |    9 |            0 |      0 |       2 |
| `naive-ui`                 |    11 |   10 |            0 |      0 |       0 |
| `voicevox`                 |    50 |    7 |            0 |      0 |       3 |
| `elk`                      |    30 |   15 |            0 |      0 |       0 |
| `misskey`                  |   224 |    6 |            0 |      0 |       0 |
| `directus`                 |   138 |   13 |            2 |      0 |       0 |
| `motion-vue`               |     1 |    0 |            0 |     57 |       0 |
| `shadcn-vue`               |    29 |    4 |            0 |     20 |       0 |
| `inspira-ui`               |     3 |    2 |            0 |     22 |       0 |
| `vue-charts`               |     0 |    0 |            0 |     12 |       0 |
| `vaul-vue`                 |     1 |    0 |            0 |      0 |       0 |
| `vee-validate`             |     0 |    0 |            0 |      0 |       2 |
| `create-vue`               |     0 |    0 |            0 |      0 |       0 |
| `vue-router`               |     3 |    0 |            0 |      0 |       1 |
| `pinia`                    |     3 |    2 |            0 |      0 |       0 |
| `vue-tui`                  |     0 |    0 |            0 |      0 |       0 |
| `vue-termui`               |    10 |   17 |            5 |      0 |       0 |
| `vue-element-admin`        |    12 |    5 |            1 |      0 |       6 |
| `element`                  |    57 |   37 |            0 |      0 |       1 |
| `lx-music-desktop`         |    68 |   16 |            0 |      0 |       3 |
| `uni-app`                  |     1 |    0 |            0 |      0 |       0 |
| `vue2-elm`                 |     4 |    0 |            0 |      0 |       1 |
| `filebrowser`              |    17 |   17 |            0 |      0 |       9 |
| `docsify`                  |     0 |    0 |            0 |      0 |       0 |
| `dashy`                    |    13 |    5 |            3 |      0 |       0 |
| `vue-devtools-v6`          |    19 |   12 |            0 |      1 |       0 |
| `vant`                     |     9 |    0 |            0 |      0 |       0 |
| `vuepress`                 |     1 |    3 |            0 |      0 |       0 |
| `automa`                   |    47 |   10 |            0 |      0 |      13 |
| `vue-pure-admin`           |    22 |    2 |            0 |      0 |       1 |
| `vue-manage-system`        |     0 |    2 |            0 |      0 |       0 |
| `vitepress`                |     2 |    1 |            0 |      0 |       0 |
| `vux`                      |     8 |    0 |            0 |      0 |       3 |
| `koel`                     |   260 |   56 |            0 |      0 |       1 |
| `better-scroll`            |     8 |    0 |            0 |      0 |       0 |
| `mint-ui`                  |    11 |    0 |            0 |      0 |       0 |
| `scalar`                   |   103 |   35 |            0 |     20 |       0 |
| `soybean-admin`            |     1 |    4 |            0 |      0 |       0 |
| `zy-player`                |    49 |    2 |            0 |      0 |       4 |
| `bootstrap-vue`            |     3 |    0 |            0 |      0 |       0 |
| `habitica`                 |   150 |  110 |            2 |      0 |       3 |
| `tiny-rdm`                 |     5 |    7 |            0 |      0 |       0 |
| `mealie`                   |    52 |   13 |            0 |      0 |       2 |
| `mall-admin-web`           |     2 |    1 |            0 |      0 |       6 |
| `douyin`                   |   109 |    0 |            0 |      0 |       0 |
| `vuestic-admin`            |    14 |    0 |            0 |      0 |       0 |
| `vue-storefront`           |     7 |    2 |            0 |      0 |       0 |
| `vue-virtual-scroller`     |     0 |    0 |            0 |      0 |       5 |
| `vue-echarts`              |     0 |    1 |            0 |      0 |       2 |
| `gridea`                   |     7 |    0 |            0 |      0 |       0 |
| `dbx`                      |   585 |  134 |            5 |      0 |     111 |
| `vue-material`             |    20 |    7 |            0 |      1 |       6 |
| `datav`                    |     0 |    0 |            0 |      0 |       0 |
| `buefy`                    |    84 |   26 |            0 |      1 |      15 |
| `cube-ui`                  |    21 |    0 |            0 |      0 |       0 |
| `youtube-dl-gui`           |    16 |    2 |            0 |      0 |       8 |
| `solidtime`                |    53 |   48 |            0 |      3 |       0 |
| `crater`                   |    92 |    1 |            0 |      0 |     103 |
| `vue-native-core`          |     0 |    0 |            0 |      0 |       0 |
| `muse-ui`                  |     0 |    0 |            0 |      0 |       0 |
| `inertia`                  |   161 |    6 |            0 |      0 |       0 |
| `gui-for-singbox`          |    18 |    6 |            0 |      0 |       4 |
| `vue-fabric-editor`        |     1 |    0 |            0 |      0 |       0 |
| `vue-grid-layout`          |     0 |    0 |            0 |      0 |       0 |
| `splayer`                  |    76 |    2 |            0 |      0 |       0 |
| `vuelidate`                |     4 |    0 |            0 |      0 |      35 |
| `vuetorrent`               |    56 |   14 |            0 |      0 |      71 |
| `vue-multiselect`          |    28 |   11 |            0 |      0 |       0 |
| `frpc-desktop`             |     0 |    0 |            0 |      0 |       0 |
| `v-charts`                 |     0 |    0 |            0 |      0 |       0 |
| `music-website`            |     5 |    3 |            0 |      0 |       0 |
| `vue-flow`                 |     7 |    2 |            0 |      0 |       0 |
| `mavon-editor`             |    17 |    0 |            0 |      0 |       0 |
| `nutui`                    |    57 |    2 |            0 |      0 |       0 |
| `nativescript-vue`         |     0 |    0 |            0 |      0 |       0 |
| `sigma-file-manager`       |    52 |    7 |            0 |      0 |       0 |
| `gogocode`                 |    13 |    6 |            2 |      0 |       2 |
| `vue-chartjs`              |     0 |    0 |            0 |      0 |       0 |
| `vuesax`                   |     7 |    7 |            0 |      0 |       0 |
| `cssgridgenerator`         |     7 |    0 |            0 |      0 |       2 |
| `varlet`                   |    32 |    4 |            0 |      0 |       4 |
| `vue-select`               |     3 |    0 |            0 |      0 |       0 |
| `vue-cropper`              |     0 |    0 |            0 |      0 |       0 |
| `vue-draggable-next`       |     0 |    0 |            0 |      0 |       0 |
| `vue-js-modal`             |     3 |    0 |            0 |      0 |       0 |
| `vue-bits`                 |    12 |    0 |            0 |      3 |       0 |
| `vue-netcore`              |    11 |    5 |            0 |      0 |       1 |
| `vue-draggable-plus`       |     0 |    0 |            0 |      0 |       0 |
| `tdesign`                  |     0 |    0 |            0 |      0 |       0 |
| `epic-spinners`            |     1 |    0 |            0 |      0 |       2 |
| `portal-vue`               |    18 |    1 |            0 |      0 |       0 |
| `vuestic-ui`               |   139 |  103 |            1 |      1 |      11 |
| `piclist`                  |    50 |    6 |            0 |      0 |      26 |
| `vue-draggable-resizable`  |     6 |    0 |            0 |      0 |       0 |
| `pinry`                    |     0 |    0 |            0 |      0 |       0 |
| `vonic`                    |     0 |    0 |            0 |      0 |       0 |
| `laravel-breeze`           |    16 |    2 |            0 |      0 |       0 |
| `frappe-crm`               |   140 |   25 |            0 |      0 |       4 |
| `v-viewer`                 |     0 |    0 |            0 |      0 |       5 |
| `antares`                  |   183 |   15 |           26 |      0 |       5 |
| `heyui`                    |    33 |    9 |            0 |      0 |       0 |
| `vue-data-ui`              |    84 |   29 |            0 |      0 |       5 |
| `splitpanes`               |     0 |    0 |            0 |      0 |       0 |
| `tailwind-config-viewer`   |     0 |    0 |            0 |      0 |       0 |
| `frappe-builder`           |   138 |   32 |            0 |      0 |       0 |
| `vue-uploader`             |     0 |    0 |            0 |      0 |       0 |
| `vant-demo`                |     0 |    0 |            0 |      0 |       0 |
| `vue-dropzone`             |     0 |    0 |            0 |      0 |       0 |
| `multiple-select`          |     1 |    0 |            0 |      0 |       1 |
| `alexandrie`               |    48 |    5 |            0 |      0 |       4 |
| `vue-fullpage-js`          |     0 |    0 |            0 |      0 |       0 |
| `arco-design-pro-vue`      |     1 |    0 |            0 |      0 |       0 |
| `vue-datepicker`           |     6 |    1 |            0 |      0 |       0 |
| `layoutit-grid`            |    15 |    4 |            2 |      0 |       0 |
| `jellyfin-vue`             |    39 |    0 |            0 |      0 |       2 |
| `lew-ui`                   |    42 |    4 |            0 |      0 |       0 |
| `vue-sonner`               |     0 |    0 |            0 |      0 |       0 |
| `element-plus-x`           |    20 |    0 |            0 |      0 |       0 |
| `vue-vine`                 |     0 |    0 |            0 |      0 |       0 |
| `vue-calendar`             |     2 |    0 |            0 |      0 |       0 |
| `prevue`                   |     2 |    1 |            0 |      0 |       0 |
| `bym-vue-echarts`          |     0 |    0 |            0 |      0 |       0 |
| `vue-apexcharts`           |     0 |    0 |            0 |      0 |       0 |
| `vue-lottie`               |     0 |    0 |            0 |      0 |       0 |
| `vue-cal-v4`               |     8 |    2 |            0 |      0 |       0 |
| `airi`                     |    33 |    3 |            0 |      1 |      21 |
| `vuefes-japan-speakers`    |     0 |    0 |            0 |      0 |       0 |
| `mobile-web-best-practice` |     3 |    0 |            0 |      0 |       0 |
| `wave-ui`                  |    28 |    6 |            0 |      0 |       0 |
| `dho-web-client`           |    12 |    2 |            0 |      0 |       9 |
| `vue3-admin-design`        |     0 |    0 |            0 |     15 |       0 |
| `vue3-antd-admin`          |     6 |    1 |            0 |     67 |       0 |
| `vue-core-vapor`           |     0 |    2 |            0 |      0 |       0 |
| `vue-jsx-vapor`            |     0 |    0 |            0 |      0 |       0 |
| `wakapi`                   |     6 |    0 |            0 |     29 |       0 |
| `petite-vue`               |     0 |    2 |            0 |      0 |       0 |
| **total sites**            |  4448 | 1172 |           54 |    255 |     532 |
| **projects seen**          |   103 |   77 |           12 |     16 |      47 |

### Dimension 4: binding_source — declaration-site presence signals (SFC file counts, NOT per-expression attribution)

| project                    | setup | props | data | inject |
| -------------------------- | ----: | ----: | ---: | -----: |
| `vue-vben-admin`           |   593 |   298 |    0 |      1 |
| `hoppscotch`               |   355 |   271 |    1 |      0 |
| `element-plus`             |   676 |   155 |    4 |     44 |
| `ant-design-vue`           |   497 |    24 |    5 |      5 |
| `reka-ui`                  |   725 |   526 |    0 |      1 |
| `primevue`                 |   616 |   269 | 1433 |     49 |
| `vuetify`                  |   831 |    62 |  527 |      0 |
| `naive-ui`                 |  1182 |    15 |    5 |      0 |
| `voicevox`                 |   133 |    94 |    0 |      6 |
| `elk`                      |   244 |   138 |    1 |      0 |
| `misskey`                  |   583 |   410 |    0 |     14 |
| `directus`                 |   535 |   446 |    8 |     23 |
| `motion-vue`               |    71 |    15 |    0 |      0 |
| `shadcn-vue`               |  6457 |  4788 |    4 |      0 |
| `inspira-ui`               |   481 |   307 |    1 |      6 |
| `vue-charts`               |   184 |    32 |    0 |      0 |
| `vaul-vue`                 |    21 |     4 |    0 |      0 |
| `vee-validate`             |    80 |    32 |    0 |      0 |
| `create-vue`               |    16 |     4 |    0 |      0 |
| `vue-router`               |    86 |     5 |    1 |      3 |
| `pinia`                    |    17 |     2 |    0 |      2 |
| `vue-tui`                  |    19 |     8 |    0 |      1 |
| `vue-termui`               |    75 |    12 |    0 |      0 |
| `vue-element-admin`        |     0 |    45 |   82 |      0 |
| `element`                  |     0 |   118 |  116 |     33 |
| `lx-music-desktop`         |    32 |    54 |   18 |      0 |
| `uni-app`                  |     9 |    28 |   39 |      0 |
| `vue2-elm`                 |     0 |    20 |   53 |      0 |
| `filebrowser`              |    40 |    25 |    2 |      8 |
| `docsify`                  |     0 |     0 |    0 |      0 |
| `dashy`                    |     0 |    49 |  139 |      0 |
| `vue-devtools-v6`          |     7 |    49 |   20 |      6 |
| `vant`                     |   110 |    12 |    9 |      0 |
| `vuepress`                 |     0 |    12 |   10 |      0 |
| `automa`                   |   199 |   179 |    0 |     22 |
| `vue-pure-admin`           |   250 |    48 |    3 |      0 |
| `vue-manage-system`        |    40 |     6 |    0 |      0 |
| `vitepress`                |    69 |    39 |    0 |      4 |
| `vux`                      |     0 |   104 |  189 |      1 |
| `koel`                     |   313 |   170 |    0 |      3 |
| `better-scroll`            |     0 |     0 |   41 |      0 |
| `mint-ui`                  |     0 |    32 |   37 |      0 |
| `scalar`                   |  1974 |  1904 |    0 |      2 |
| `soybean-admin`            |    89 |    30 |    0 |      0 |
| `zy-player`                |     0 |     0 |   12 |      0 |
| `bootstrap-vue`            |     0 |     3 |   11 |      0 |
| `habitica`                 |     0 |   181 |  263 |      0 |
| `tiny-rdm`                 |   140 |   119 |    0 |      0 |
| `mealie`                   |   204 |   104 |    0 |      0 |
| `mall-admin-web`           |    83 |    18 |    0 |      4 |
| `douyin`                   |    88 |    61 |   31 |      1 |
| `vuestic-admin`            |    83 |    37 |    0 |      1 |
| `vue-storefront`           |     0 |     5 |    4 |      0 |
| `vue-virtual-scroller`     |     8 |     6 |    7 |      0 |
| `vue-echarts`              |    15 |     3 |    0 |      0 |
| `gridea`                   |     0 |     0 |    0 |      0 |
| `dbx`                      |   375 |   358 |    0 |      2 |
| `vue-material`             |     0 |   117 |  158 |     29 |
| `datav`                    |     0 |    72 |   74 |      0 |
| `buefy`                    |    85 |    83 |  288 |      8 |
| `cube-ui`                  |     0 |    75 |  105 |      6 |
| `youtube-dl-gui`           |    81 |    46 |    0 |      0 |
| `solidtime`                |   383 |   306 |    4 |      2 |
| `crater`                   |   249 |   132 |    0 |     54 |
| `vue-native-core`          |     0 |     0 |    0 |      0 |
| `muse-ui`                  |     0 |     0 |    3 |      0 |
| `inertia`                  |   450 |   209 |    1 |      1 |
| `gui-for-singbox`          |    96 |    60 |    0 |     16 |
| `vue-fabric-editor`        |    68 |    14 |    1 |      6 |
| `vue-grid-layout`          |     0 |     4 |    1 |      1 |
| `splayer`                  |   153 |    77 |    0 |      1 |
| `vuelidate`                |     0 |     2 |   16 |      0 |
| `vuetorrent`               |   144 |    96 |    0 |      0 |
| `vue-multiselect`          |     0 |     2 |   16 |      0 |
| `frpc-desktop`             |    12 |     1 |    0 |      0 |
| `v-charts`                 |     0 |     1 |   18 |      0 |
| `music-website`            |    45 |    10 |    0 |      0 |
| `vue-flow`                 |   178 |    81 |    1 |      3 |
| `mavon-editor`             |     0 |     3 |    4 |      0 |
| `nutui`                    |   888 |   223 |    2 |     23 |
| `nativescript-vue`         |    13 |     1 |    1 |      0 |
| `sigma-file-manager`       |   295 |   196 |    0 |      2 |
| `gogocode`                 |     0 |   135 |  156 |      1 |
| `vue-chartjs`              |     1 |     0 |    9 |      0 |
| `vuesax`                   |     0 |    50 |   35 |      1 |
| `cssgridgenerator`         |     0 |     0 |    4 |      0 |
| `varlet`                   |   100 |    22 |    1 |      0 |
| `vue-select`               |     0 |     1 |    2 |      0 |
| `vue-cropper`              |     0 |     2 |    0 |      0 |
| `vue-draggable-next`       |     0 |     4 |   26 |      0 |
| `vue-js-modal`             |     0 |     6 |    8 |      0 |
| `vue-bits`                 |   320 |   302 |    1 |      0 |
| `vue-netcore`              |    77 |    35 |   20 |      0 |
| `vue-draggable-plus`       |    36 |     4 |    0 |      0 |
| `tdesign`                  |     0 |    35 |   48 |      0 |
| `epic-spinners`            |     0 |    23 |   18 |      0 |
| `portal-vue`               |     0 |     9 |   19 |      1 |
| `vuestic-ui`               |   383 |   215 |  340 |      0 |
| `piclist`                  |    56 |    32 |    0 |      0 |
| `vue-draggable-resizable`  |     6 |     1 |   41 |      0 |
| `pinry`                    |     0 |    14 |   20 |      0 |
| `vonic`                    |     0 |    24 |   55 |      0 |
| `laravel-breeze`           |    48 |    32 |    0 |      0 |
| `frappe-crm`               |   228 |   147 |    0 |     27 |
| `v-viewer`                 |     0 |     1 |    0 |      0 |
| `antares`                  |    92 |    70 |    0 |      0 |
| `heyui`                    |     0 |    74 |   64 |      2 |
| `vue-data-ui`              |   389 |   258 |    0 |      0 |
| `splitpanes`               |     7 |     3 |    0 |      1 |
| `tailwind-config-viewer`   |     0 |    27 |   10 |      1 |
| `frappe-builder`           |   150 |    95 |    3 |      6 |
| `vue-uploader`             |     0 |     3 |    4 |      0 |
| `vant-demo`                |     4 |     1 |    3 |      0 |
| `vue-dropzone`             |     0 |     5 |   10 |      0 |
| `multiple-select`          |     1 |     3 |   45 |      0 |
| `alexandrie`               |   193 |   102 |    0 |      0 |
| `vue-fullpage-js`          |     0 |     1 |    2 |      0 |
| `arco-design-pro-vue`      |    84 |    14 |    0 |      1 |
| `vue-datepicker`           |    19 |    17 |    0 |      0 |
| `layoutit-grid`            |    69 |    65 |    0 |      0 |
| `jellyfin-vue`             |   132 |    63 |    0 |      3 |
| `lew-ui`                   |   402 |    99 |    0 |      5 |
| `vue-sonner`               |    20 |    10 |    0 |      0 |
| `element-plus-x`           |   282 |    55 |    0 |      0 |
| `vue-vine`                 |     2 |     0 |    0 |      0 |
| `vue-calendar`             |     0 |     1 |    2 |      0 |
| `prevue`                   |     0 |     3 |    9 |      0 |
| `bym-vue-echarts`          |     0 |    16 |   37 |      0 |
| `vue-apexcharts`           |     0 |     0 |    0 |      0 |
| `vue-lottie`               |     0 |     1 |    2 |      0 |
| `vue-cal-v4`               |     0 |    11 |   12 |      5 |
| `airi`                     |   569 |   240 |    0 |      8 |
| `vuefes-japan-speakers`    |    14 |    11 |    0 |      0 |
| `mobile-web-best-practice` |     0 |     0 |    0 |      0 |
| `wave-ui`                  |     3 |    67 |  118 |      4 |
| `dho-web-client`           |     0 |   161 |  106 |      0 |
| `vue3-admin-design`        |     7 |     0 |    0 |      0 |
| `vue3-antd-admin`          |    92 |    35 |    0 |      0 |
| `vue-core-vapor`           |   103 |    11 |    0 |      0 |
| `vue-jsx-vapor`            |     0 |     0 |    0 |      0 |
| `wakapi`                   |     0 |     0 |    0 |      0 |
| `petite-vue`               |     0 |     0 |    0 |      0 |
| **total sites**            | 25634 | 16163 | 4999 |    460 |
| **projects seen**          |    89 |   127 |   79 |     51 |

### Dimension 5: block_combination (SFCs whose top-level blocks match the combination exactly)

| project                    | template-only | template-script-setup | template-script | template-both-scripts | template-script-setup-style-scoped |
| -------------------------- | ------------: | --------------------: | --------------: | --------------------: | ---------------------------------: |
| `vue-vben-admin`           |            18 |                   558 |               0 |                     0 |                                 27 |
| `hoppscotch`               |             7 |                   310 |               3 |                     1 |                                 34 |
| `element-plus`             |            81 |                   508 |              21 |                     0 |                                115 |
| `ant-design-vue`           |           119 |                   424 |              67 |                     0 |                                 67 |
| `reka-ui`                  |             4 |                   286 |               0 |                   428 |                                  5 |
| `primevue`                 |           260 |                   614 |            1588 |                     0 |                                  0 |
| `vuetify`                  |           376 |                   275 |              32 |                   495 |                                 18 |
| `naive-ui`                 |           428 |                  1134 |              31 |                     1 |                                  1 |
| `voicevox`                 |             0 |                    22 |               0 |                     2 |                                104 |
| `elk`                      |            14 |                   225 |               0 |                     1 |                                  8 |
| `misskey`                  |             0 |                   128 |               0 |                     2 |                                 32 |
| `directus`                 |             4 |                    80 |              25 |                     2 |                                439 |
| `motion-vue`               |             1 |                    42 |               0 |                     2 |                                 20 |
| `shadcn-vue`               |            57 |                  6348 |               0 |                   101 |                                  5 |
| `inspira-ui`               |            29 |                   433 |               0 |                     0 |                                 42 |
| `vue-charts`               |             9 |                   180 |               0 |                     0 |                                  3 |
| `vaul-vue`                 |             0 |                     8 |               0 |                     0 |                                 13 |
| `vee-validate`             |             0 |                    63 |               0 |                     0 |                                 14 |
| `create-vue`               |            20 |                     6 |               0 |                     0 |                                 10 |
| `vue-router`               |            13 |                    80 |               2 |                     2 |                                  3 |
| `pinia`                    |             7 |                     8 |               0 |                     0 |                                  2 |
| `vue-tui`                  |             0 |                    19 |               0 |                     0 |                                  0 |
| `vue-termui`               |             2 |                    74 |               0 |                     0 |                                  1 |
| `vue-element-admin`        |             9 |                     0 |              48 |                     0 |                                  0 |
| `element`                  |             0 |                     0 |             113 |                     0 |                                  0 |
| `lx-music-desktop`         |             2 |                     1 |              12 |                     0 |                                  0 |
| `uni-app`                  |             0 |                     2 |              10 |                     0 |                                  1 |
| `vue2-elm`                 |             0 |                     0 |               0 |                     0 |                                  0 |
| `filebrowser`              |             0 |                    25 |              18 |                     0 |                                 11 |
| `docsify`                  |             0 |                     0 |               0 |                     0 |                                  0 |
| `dashy`                    |             1 |                     0 |               7 |                     0 |                                  0 |
| `vue-devtools-v6`          |             8 |                     5 |              77 |                     1 |                                  1 |
| `vant`                     |             2 |                    52 |               2 |                     0 |                                  0 |
| `vuepress`                 |             1 |                     0 |               7 |                     0 |                                  0 |
| `automa`                   |             0 |                   139 |               7 |                     1 |                                 12 |
| `vue-pure-admin`           |             1 |                   153 |               1 |                     0 |                                 90 |
| `vue-manage-system`        |             0 |                     6 |               0 |                     0 |                                 24 |
| `vitepress`                |            21 |                    11 |               0 |                     0 |                                 57 |
| `vux`                      |             6 |                     0 |             120 |                     0 |                                  0 |
| `koel`                     |            16 |                   189 |               0 |                     0 |                                117 |
| `better-scroll`            |             0 |                     0 |               6 |                     0 |                                  0 |
| `mint-ui`                  |             1 |                     0 |               5 |                     0 |                                  0 |
| `scalar`                   |             9 |                   181 |               5 |                  1632 |                                119 |
| `soybean-admin`            |             0 |                     2 |               0 |                     0 |                                 85 |
| `zy-player`                |             0 |                     0 |               6 |                     0 |                                  0 |
| `bootstrap-vue`            |             7 |                     0 |               5 |                     0 |                                  0 |
| `habitica`                 |             4 |                     0 |              58 |                     0 |                                  0 |
| `tiny-rdm`                 |             0 |                     2 |               0 |                     0 |                                126 |
| `mealie`                   |             1 |                   128 |               0 |                     0 |                                 42 |
| `mall-admin-web`           |             0 |                     3 |               0 |                     0 |                                 59 |
| `douyin`                   |             0 |                     5 |               1 |                     0 |                                 78 |
| `vuestic-admin`            |             5 |                    60 |               1 |                     0 |                                 12 |
| `vue-storefront`           |             0 |                     0 |               7 |                     0 |                                  0 |
| `vue-virtual-scroller`     |             0 |                     4 |               0 |                     0 |                                  2 |
| `vue-echarts`              |             0 |                    10 |               0 |                     0 |                                  2 |
| `gridea`                   |             0 |                     0 |               0 |                     0 |                                  0 |
| `dbx`                      |             0 |                   284 |               0 |                     1 |                                 73 |
| `vue-material`             |             0 |                     0 |             128 |                     0 |                                  0 |
| `datav`                    |             0 |                     0 |              38 |                     0 |                                  0 |
| `buefy`                    |            13 |                    78 |             308 |                     0 |                                  7 |
| `cube-ui`                  |             0 |                     0 |              24 |                     0 |                                  0 |
| `youtube-dl-gui`           |             2 |                    69 |               0 |                     0 |                                 11 |
| `solidtime`                |             6 |                   262 |               0 |                     0 |                                115 |
| `crater`                   |            38 |                   244 |              19 |                     0 |                                  1 |
| `vue-native-core`          |             0 |                     0 |               0 |                     0 |                                  0 |
| `muse-ui`                  |             0 |                     0 |               1 |                     0 |                                  0 |
| `inertia`                  |             8 |                   438 |               1 |                     2 |                                  0 |
| `gui-for-singbox`          |             2 |                    63 |               0 |                     0 |                                 30 |
| `vue-fabric-editor`        |             1 |                     3 |               0 |                     1 |                                 60 |
| `vue-grid-layout`          |             0 |                     0 |               0 |                     0 |                                  0 |
| `splayer`                  |             1 |                    32 |               0 |                     0 |                                115 |
| `vuelidate`                |             0 |                     0 |              21 |                     0 |                                  0 |
| `vuetorrent`               |             1 |                    96 |               0 |                     0 |                                 43 |
| `vue-multiselect`          |             0 |                     0 |              25 |                     0 |                                  0 |
| `frpc-desktop`             |             0 |                     5 |               1 |                     0 |                                  7 |
| `v-charts`                 |             1 |                     0 |              26 |                     0 |                                  0 |
| `music-website`            |             1 |                     4 |               0 |                     0 |                                 39 |
| `vue-flow`                 |             2 |                   106 |              12 |                    40 |                                  3 |
| `mavon-editor`             |             0 |                     0 |               1 |                     0 |                                  0 |
| `nutui`                    |           181 |                   794 |             132 |                     0 |                                 13 |
| `nativescript-vue`         |             3 |                    10 |               1 |                     0 |                                  1 |
| `sigma-file-manager`       |             1 |                    42 |               0 |                     0 |                                 91 |
| `gogocode`                 |             3 |                     0 |              42 |                     0 |                                  0 |
| `vue-chartjs`              |             0 |                     1 |              10 |                     0 |                                  0 |
| `vuesax`                   |             0 |                     0 |              56 |                     0 |                                  0 |
| `cssgridgenerator`         |             0 |                     0 |               0 |                     0 |                                  0 |
| `varlet`                   |             5 |                    51 |              17 |                     0 |                                 34 |
| `vue-select`               |             2 |                     0 |               0 |                     0 |                                  0 |
| `vue-cropper`              |             0 |                     0 |               0 |                     0 |                                  0 |
| `vue-draggable-next`       |             0 |                     0 |               8 |                     0 |                                  0 |
| `vue-js-modal`             |             0 |                     0 |               6 |                     0 |                                  0 |
| `vue-bits`                 |             2 |                   258 |               1 |                     6 |                                 49 |
| `vue-netcore`              |             7 |                    34 |               9 |                     0 |                                 36 |
| `vue-draggable-plus`       |             0 |                    24 |               0 |                     0 |                                  6 |
| `tdesign`                  |             9 |                     0 |              31 |                     0 |                                  0 |
| `epic-spinners`            |             0 |                     0 |              20 |                     0 |                                  0 |
| `portal-vue`               |             2 |                     0 |              25 |                     0 |                                  0 |
| `vuestic-ui`               |           194 |                   172 |             301 |                     4 |                                 62 |
| `piclist`                  |             0 |                    29 |               0 |                    14 |                                  7 |
| `vue-draggable-resizable`  |             0 |                     6 |              45 |                     0 |                                  0 |
| `pinry`                    |             0 |                     0 |               6 |                     0 |                                  0 |
| `vonic`                    |            11 |                     0 |              63 |                     0 |                                  0 |
| `laravel-breeze`           |             6 |                    48 |               0 |                     0 |                                  0 |
| `frappe-crm`               |           111 |                   204 |               0 |                     2 |                                 20 |
| `v-viewer`                 |             0 |                     0 |               3 |                     0 |                                  0 |
| `antares`                  |             0 |                    35 |               0 |                     0 |                                 39 |
| `heyui`                    |             0 |                     0 |              76 |                     0 |                                  0 |
| `vue-data-ui`              |            10 |                   238 |               0 |                     0 |                                142 |
| `splitpanes`               |             2 |                     2 |               0 |                     0 |                                  0 |
| `tailwind-config-viewer`   |             0 |                     0 |              22 |                     0 |                                  0 |
| `frappe-builder`           |             9 |                   137 |               0 |                     0 |                                  8 |
| `vue-uploader`             |             0 |                     0 |               0 |                     0 |                                  0 |
| `vant-demo`                |             2 |                     0 |               0 |                     0 |                                  0 |
| `vue-dropzone`             |             1 |                     0 |               7 |                     0 |                                  0 |
| `multiple-select`          |             0 |                     0 |               7 |                     0 |                                  1 |
| `alexandrie`               |             3 |                    33 |               0 |                     0 |                                158 |
| `vue-fullpage-js`          |             0 |                     0 |               1 |                     0 |                                  0 |
| `arco-design-pro-vue`      |             0 |                     7 |               0 |                     0 |                                 59 |
| `vue-datepicker`           |             0 |                    19 |               0 |                     0 |                                  0 |
| `layoutit-grid`            |            35 |                     4 |               0 |                     0 |                                 63 |
| `jellyfin-vue`             |             2 |                    91 |               0 |                     5 |                                 31 |
| `lew-ui`                   |            23 |                   251 |               0 |                     0 |                                141 |
| `vue-sonner`               |             6 |                    16 |               0 |                     0 |                                  2 |
| `element-plus-x`           |            42 |                    92 |               0 |                     0 |                                190 |
| `vue-vine`                 |             0 |                     2 |               0 |                     0 |                                  0 |
| `vue-calendar`             |             0 |                     0 |               0 |                     0 |                                  0 |
| `prevue`                   |             0 |                     0 |               0 |                     0 |                                  0 |
| `bym-vue-echarts`          |             0 |                     0 |               0 |                     0 |                                  0 |
| `vue-apexcharts`           |             0 |                     0 |              10 |                     0 |                                  0 |
| `vue-lottie`               |             0 |                     0 |               1 |                     0 |                                  0 |
| `vue-cal-v4`               |             0 |                     0 |               6 |                     0 |                                  0 |
| `airi`                     |            12 |                   480 |               0 |                     0 |                                 75 |
| `vuefes-japan-speakers`    |             1 |                    14 |               0 |                     0 |                                  0 |
| `mobile-web-best-practice` |             0 |                     0 |               0 |                     0 |                                  0 |
| `wave-ui`                  |             7 |                     0 |             116 |                     0 |                                  0 |
| `dho-web-client`           |             0 |                     0 |              65 |                     0 |                                  0 |
| `vue3-admin-design`        |             0 |                     0 |               0 |                     0 |                                  7 |
| `vue3-antd-admin`          |             1 |                    43 |               1 |                     0 |                                 43 |
| `vue-core-vapor`           |             0 |                    92 |               1 |                     0 |                                  0 |
| `vue-jsx-vapor`            |             0 |                     0 |               0 |                     0 |                                  0 |
| `wakapi`                   |             0 |                     0 |               0 |                     0 |                                  0 |
| `petite-vue`               |             0 |                     0 |               0 |                     0 |                                  0 |
| **total sites**            |          2312 |                 17716 |            3979 |                  2746 |                               3583 |
| **projects seen**          |            75 |                    85 |              72 |                    23 |                                 74 |

## Skipped (not mechanically derived by this scan)

- **binding_source per-expression attribution** — mapping each template identifier to its declaration site needs scope analysis (the croquis engine's job). The table above reports file-level declaration-site signals only (`<script setup>` present / `defineProps`-or-`props:` / `data()` / `inject`); the `global` source has no mechanical signal and is not measured at all.
- **`v-slot` / `#` shorthand** — scanned (22765 occurrences across hydrated projects) but reported nowhere above: the taxonomy has no `v-slot` directive row today.
- **JSX plain props** — every JSX prop is an expression binding; counting them all as `v-bind` would be noise, so only `v-*` props and `on[A-Z]*` event props (counted as `v-on`, with `_modifier` suffixes matched to modifier classes) are classified.
- **petite-vue built-ins** — `v-scope` / `v-effect` have no taxonomy row and land in `custom` (the not-in-builtin-set escape hatch).
- **Lexical limits** — pug templates are scanned line-heuristically (no pug parse); wakapi's HTML interleaves Go `{{ }}` template actions that the scanner skims over; TSX start tags reuse an HTML regex (single-uppercase-letter names are dropped as probable type parameters, other generics can leak); SVG/MathML descendants count via a fixed unambiguous-name set, so namespace children whose names collide with HTML tags count as `native`; unknown `v-on` modifier tokens (custom key aliases) are ignored.
- **Element kinds in scripts** — render functions and template strings inside `.js`/`.ts` sources are not scanned; only the file classes in the scan-scope table are.

## Scope proof (assurance rule: empty means proven-empty, never silently partial)

- **Hydrated: 142 of 142 manifest projects.**

All manifest projects were hydrated for this run: zeros above are proven-empty over the whole registered corpus.
