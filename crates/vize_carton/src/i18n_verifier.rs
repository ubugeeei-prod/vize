//! `<code>.description` for the stage verifiers' invariant codes: S2's
//! `ViolationCode` (`vize_s2::verify`) and S3's (`vize_impeto::verify`).
//! These report compiler defects, never user mistakes, so each description
//! states the broken invariant plainly. Registered by
//! [`crate::i18n_cross_file`].

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "S2V001.description",
        "An S2 span runs backwards: it starts after it ends",
        "S2 のスパンが逆向きです。開始位置が終了位置より後ろにあります",
        "S2 的区间方向颠倒：起点位于终点之后",
    ),
    (
        "S2V002.description",
        "A nested S2 line's span escapes its owner's span",
        "入れ子になった S2 の行のスパンが、所有者のスパンからはみ出しています",
        "嵌套的 S2 行的区间超出了其所有者的区间",
    ),
    (
        "S2V003.description",
        "An S2 side table references a node the artifact does not number",
        "S2 のサイドテーブルが、成果物に番号の振られていないノードを参照しています",
        "S2 旁表引用了产物中未编号的节点",
    ),
    (
        "S2V004.description",
        "A `ui.if` owns no branch",
        "`ui.if` が分岐を 1 つも持っていません",
        "`ui.if` 没有任何分支",
    ),
    (
        "S2V005.description",
        "The leading branch of a `ui.if` is unconditional",
        "`ui.if` の先頭の分岐が無条件になっています",
        "`ui.if` 的首个分支是无条件的",
    ),
    (
        "S2V006.description",
        "An unconditional `ui.if` branch is not the last branch",
        "無条件の `ui.if` 分岐が最後の分岐になっていません",
        "无条件的 `ui.if` 分支不是最后一个分支",
    ),
    (
        "S3V001.description",
        "Two S3 ops, regions or effects share an id",
        "S3 の op・リージョン・エフェクトの ID が重複しています",
        "S3 的 op、区域或 effect 的 id 重复",
    ),
    (
        "S3V002.description",
        "The S3 root region `r#0` is missing, or has a parent or owner",
        "S3 のルートリージョン `r#0` がないか、親や所有者を持っています",
        "S3 根区域 `r#0` 缺失，或带有父级或所有者",
    ),
    (
        "S3V003.description",
        "An S3 op references a missing region",
        "S3 の op が存在しないリージョンを参照しています",
        "S3 的 op 引用了不存在的区域",
    ),
    (
        "S3V004.description",
        "An S3 state edge's source or target does not resolve",
        "S3 の状態エッジの始点か終点を解決できません",
        "S3 状态边的起点或终点无法解析",
    ),
    (
        "S3V005.description",
        "An S3 region's parent or owner does not resolve, or a non-root region lacks one",
        "S3 のリージョンの親か所有者を解決できないか、ルート以外のリージョンにそれがありません",
        "S3 区域的父级或所有者无法解析，或非根区域缺少它们",
    ),
    (
        "S3V006.description",
        "An S3 region or op escapes its parent, owner or containing region, or regions nest in a cycle",
        "S3 のリージョンや op が親・所有者・所属リージョンからはみ出しているか、リージョンの親子関係が循環しています",
        "S3 的区域或 op 超出了其父级、所有者或所在区域，或区域的父子关系形成了循环",
    ),
    (
        "S3V007.description",
        "An S3 effect or state edge leaves its effect scope, or references a missing one",
        "S3 のエフェクトや状態エッジがエフェクトスコープから外れているか、存在しないスコープを参照しています",
        "S3 的 effect 或状态边离开了其 effect 作用域，或引用了不存在的作用域",
    ),
    (
        "S3V008.description",
        "An S3 scheduled edge points backward or to itself",
        "S3 のスケジュール済みエッジが後方か自分自身を指しています",
        "S3 的调度边指向前面或指向自身",
    ),
    (
        "S3V009.description",
        "An S3 op has an invalid operand",
        "S3 の op に不正なオペランドがあります",
        "S3 的 op 含有无效的操作数",
    ),
];
