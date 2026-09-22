//! Catalog of S3 verifier codes (`vize_impeto::verify::ViolationCode`).
//!
//! Each code has `s3/<code>.message` and `s3/<code>.help` in en, ja and
//! zh. The verifier still prints its call-site message; these entries name
//! the invariant. `tests/tooling/davinci-diagnostic-catalog.test.ts` reads
//! `as_str()` from source and fails when a code lacks either entry.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every S3 verifier entry into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// `(key, en, ja, zh)` for `ViolationCode::as_str`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "s3/S3V001.message",
        "An op, region, or effect id is repeated.",
        "op、region、effect の id が重複しています。",
        "op、region 或 effect 的 id 重复了。",
    ),
    (
        "s3/S3V001.help",
        "Give every op, region, and effect its own id.",
        "op、region、effect にはそれぞれ別の id を付けてください。",
        "请为每个 op、region 和 effect 分配不同的 id。",
    ),
    (
        "s3/S3V002.message",
        "The program has no root region, or the root names a parent or owner.",
        "ルート region がないか、ルートが parent または owner を持っています。",
        "缺少根 region，或根 region 带有 parent 或 owner。",
    ),
    (
        "s3/S3V002.help",
        "Region r#0 must exist and must not name a parent or an owner.",
        "region r#0 は存在し、parent も owner も持たない必要があります。",
        "region r#0 必须存在，且不能指定 parent 或 owner。",
    ),
    (
        "s3/S3V003.message",
        "An op names a region that does not exist.",
        "op が存在しない region を指しています。",
        "op 指向了不存在的 region。",
    ),
    (
        "s3/S3V003.help",
        "Point the op at a region in this program.",
        "op の region はこのプログラムにある region にしてください。",
        "请让 op 的 region 指向本程序中的 region。",
    ),
    (
        "s3/S3V004.message",
        "A state edge names an endpoint that does not resolve.",
        "state edge の端点が解決できません。",
        "state edge 的端点无法解析。",
    ),
    (
        "s3/S3V004.help",
        "Both ends of a state edge must be ops in this program.",
        "state edge の両端はこのプログラムの op である必要があります。",
        "state edge 的两端都必须是本程序中的 op。",
    ),
    (
        "s3/S3V005.message",
        "A region names a parent or owner that does not resolve, or a non-root region omits them.",
        "region の parent または owner が解決できないか、ルート以外がそれらを欠いています。",
        "region 的 parent 或 owner 无法解析，或非根 region 没有指定它们。",
    ),
    (
        "s3/S3V005.help",
        "Every region except r#0 must name a parent region and an owner op that exist.",
        "r#0 以外の region は、存在する parent region と owner op を指定してください。",
        "除 r#0 外，每个 region 都必须指定存在的 parent region 和 owner op。",
    ),
    (
        "s3/S3V006.message",
        "A region, op, or effect escapes the region that contains it, or regions form a parent cycle.",
        "region、op、effect が包含 region の外に出ているか、parent が循環しています。",
        "region、op 或 effect 超出了包含它的 region，或 parent 形成了环。",
    ),
    (
        "s3/S3V006.help",
        "Keep each span inside its parent and owner, and do not cycle parent links.",
        "各スパンを parent と owner の内側に保ち、parent の循環を作らないでください。",
        "请把每个 span 保持在 parent 和 owner 之内，不要让 parent 成环。",
    ),
    (
        "s3/S3V007.message",
        "An effect names a missing scope, owner, or region, or a state edge leaves its effect scope.",
        "effect が欠けた scope、owner、region を指すか、state edge が effect scope の外に出ています。",
        "effect 指向了不存在的 scope、owner 或 region，或 state edge 离开了 effect scope。",
    ),
    (
        "s3/S3V007.help",
        "Keep the effect and its edges inside the region and effect scope it names.",
        "effect とその edge を、指している region と effect scope の内側に保ってください。",
        "请把 effect 及其 edge 保持在它指定的 region 和 effect scope 之内。",
    ),
    (
        "s3/S3V008.message",
        "A scheduled edge points backward or at itself.",
        "スケジュールされた edge が後方または自分自身を指しています。",
        "被调度的 edge 指向了后方或它自己。",
    ),
    (
        "s3/S3V008.help",
        "Schedule edges toward a later op.",
        "edge はより後ろの op に向けてスケジュールしてください。",
        "请把 edge 调度到更靠后的 op。",
    ),
    (
        "s3/S3V009.message",
        "An operand is not valid for its op.",
        "operand がその op に対して無効です。",
        "operand 对其 op 无效。",
    ),
    (
        "s3/S3V009.help",
        "Use an operand the op's verifier accepts.",
        "その op の検証が受け付ける operand を使ってください。",
        "请使用该 op 的验证器接受的 operand。",
    ),
    (
        "s3/S3V010.message",
        "A placement alternative or committed choice does not match the placement rules.",
        "placement の候補または採用結果が placement 規則と一致しません。",
        "placement 的候选或已提交选择不符合 placement 规则。",
    ),
    (
        "s3/S3V010.help",
        "Re-derive the placement; a hoist must not nest, and a group must stay contiguous.",
        "placement を再導出してください。hoist は入れ子にせず、group は連続させてください。",
        "请重新推导 placement。hoist 不能嵌套，group 必须保持连续。",
    ),
];
