//! The attached-binding printers, split from [`print`](super) along the
//! op-family boundary when `ui.bind`/`ui.on` grew the family past the
//! source budget. The grammar per line mirrors the parse side
//! (`folio/parse/binding_line.rs`): optional fields in fixed order, each
//! separated by exactly one space.

use core::fmt::{Result, Write};

use super::super::owned::{
    FolioAttribute, FolioBind, FolioBinding, FolioOn, FolioSlotContent, FolioVueCssBind,
    FolioVueDirective, FolioVueMemo, FolioVueOnce, FolioVueSlotScope, FolioVueSync,
};
use super::{end_line, indent, print_expr, print_name, quoted};
use vize_davinci::folio::FolioMode;
use vize_s0::String;

pub(super) fn print_attribute<W: Write>(
    w: &mut W,
    attribute: &FolioAttribute,
    depth: usize,
    mode: FolioMode,
) -> Result {
    indent(w, depth)?;
    write!(w, "attr {}", attribute.name)?;
    if let Some(value) = &attribute.value {
        w.write_char('=')?;
        quoted(w, value.as_str())?;
    }
    end_line(w, attribute.span, mode)
}

pub(super) fn print_binding<W: Write>(
    w: &mut W,
    binding: &FolioBinding,
    depth: usize,
    mode: FolioMode,
) -> Result {
    match binding {
        FolioBinding::Bind(bind) => print_bind(w, bind, depth, mode),
        FolioBinding::On(on) => print_on(w, on, depth, mode),
        FolioBinding::Model(model) => {
            indent(w, depth)?;
            w.write_str("ui.model")?;
            if let Some(argument) = &model.argument {
                w.write_str(" name=")?;
                print_name(w, argument, mode)?;
            }
            w.write_str(" read=")?;
            print_expr(w, &model.contract.read, mode)?;
            w.write_str(" write=")?;
            print_expr(w, &model.contract.write, mode)?;
            end_line(w, model.span, mode)?;
            for attribute in &model.attributes {
                print_attribute(w, attribute, depth + 1, mode)?;
            }
            Ok(())
        }
        FolioBinding::SlotContent(content) => print_slot_content(w, content, depth, mode),
        FolioBinding::VueDirective(directive) => print_directive(w, directive, depth, mode),
        FolioBinding::VueCssBind(bind) => print_css_bind(w, bind, depth, mode),
        FolioBinding::VueSync(sync) => print_sync(w, sync, depth, mode),
        FolioBinding::VueSlotScope(scope) => print_slot_scope(w, scope, depth, mode),
        FolioBinding::VueOnce(once) => print_once(w, once, depth, mode),
        FolioBinding::VueMemo(memo) => print_memo(w, memo, depth, mode),
    }
}

fn print_mods<W: Write>(w: &mut W, modifiers: &[String]) -> Result {
    if modifiers.is_empty() {
        return Ok(());
    }
    if modifiers
        .iter()
        .all(|modifier| !modifier.as_str().contains([',', '"', '\\', '\n', '\r']))
    {
        w.write_str(" mods=\"")?;
        for (i, modifier) in modifiers.iter().enumerate() {
            if i > 0 {
                w.write_char(',')?;
            }
            w.write_str(modifier.as_str())?;
        }
        return w.write_char('"');
    }
    w.write_str(" mods=[")?;
    for (i, modifier) in modifiers.iter().enumerate() {
        if i > 0 {
            w.write_char(',')?;
        }
        quoted(w, modifier.as_str())?;
    }
    w.write_char(']')
}

fn print_bind<W: Write>(w: &mut W, bind: &FolioBind, depth: usize, mode: FolioMode) -> Result {
    indent(w, depth)?;
    w.write_str("ui.bind")?;
    if let Some(name) = &bind.name {
        w.write_str(" name=")?;
        print_name(w, name, mode)?;
    }
    print_mods(w, &bind.modifiers)?;
    if let Some(value) = &bind.value {
        w.write_str(" value=")?;
        print_expr(w, value, mode)?;
    }
    end_line(w, bind.span, mode)
}

fn print_on<W: Write>(w: &mut W, on: &FolioOn, depth: usize, mode: FolioMode) -> Result {
    indent(w, depth)?;
    w.write_str("ui.on")?;
    if let Some(name) = &on.name {
        w.write_str(" name=")?;
        print_name(w, name, mode)?;
    }
    print_mods(w, &on.modifiers)?;
    if let Some(handler) = &on.handler {
        w.write_str(" handler=")?;
        print_expr(w, handler, mode)?;
    }
    end_line(w, on.span, mode)
}

fn print_slot_content<W: Write>(
    w: &mut W,
    content: &FolioSlotContent,
    depth: usize,
    mode: FolioMode,
) -> Result {
    indent(w, depth)?;
    w.write_str("ui.slot-content")?;
    if let Some(name) = &content.name {
        w.write_str(" name=")?;
        print_name(w, name, mode)?;
    }
    print_mods(w, &content.modifiers)?;
    if let Some(params) = &content.params {
        w.write_str(" params=")?;
        print_expr(w, params, mode)?;
    }
    end_line(w, content.span, mode)
}

fn print_directive<W: Write>(
    w: &mut W,
    directive: &FolioVueDirective,
    depth: usize,
    mode: FolioMode,
) -> Result {
    indent(w, depth)?;
    w.write_str("vue.directive ")?;
    quoted(w, directive.name.as_str())?;
    if let Some(argument) = &directive.argument {
        w.write_str(" arg=")?;
        print_name(w, argument, mode)?;
    }
    print_mods(w, &directive.modifiers)?;
    if let Some(value) = &directive.value {
        w.write_str(" value=")?;
        print_expr(w, value, mode)?;
    }
    end_line(w, directive.span, mode)
}

fn print_css_bind<W: Write>(
    w: &mut W,
    bind: &FolioVueCssBind,
    depth: usize,
    mode: FolioMode,
) -> Result {
    indent(w, depth)?;
    w.write_str("vue.css-bind value=")?;
    print_expr(w, &bind.value, mode)?;
    end_line(w, bind.span, mode)
}

fn print_sync<W: Write>(w: &mut W, sync: &FolioVueSync, depth: usize, mode: FolioMode) -> Result {
    indent(w, depth)?;
    w.write_str("vue.sync name=")?;
    quoted(w, sync.name.as_str())?;
    print_mods(w, &sync.modifiers)?;
    w.write_str(" value=")?;
    print_expr(w, &sync.value, mode)?;
    end_line(w, sync.span, mode)
}

fn print_slot_scope<W: Write>(
    w: &mut W,
    scope: &FolioVueSlotScope,
    depth: usize,
    mode: FolioMode,
) -> Result {
    indent(w, depth)?;
    w.write_str("vue.slot-scope")?;
    if let Some(name) = &scope.name {
        w.write_str(" name=")?;
        quoted(w, name.as_str())?;
    }
    if let Some(params) = &scope.params {
        w.write_str(" params=")?;
        print_expr(w, params, mode)?;
    }
    end_line(w, scope.span, mode)
}

fn print_once<W: Write>(w: &mut W, once: &FolioVueOnce, depth: usize, mode: FolioMode) -> Result {
    indent(w, depth)?;
    w.write_str("vue.once")?;
    end_line(w, once.span, mode)
}

fn print_memo<W: Write>(w: &mut W, memo: &FolioVueMemo, depth: usize, mode: FolioMode) -> Result {
    indent(w, depth)?;
    w.write_str("vue.memo value=")?;
    print_expr(w, &memo.value, mode)?;
    end_line(w, memo.span, mode)
}
