use vize_carton::{String, cstr};
use vize_relief::{ElementNode, ElementType, Namespace, TemplateChildNode};

use crate::virtual_ts::component_reference::resolved_component_binding_reference;

use super::ComponentSlotCheckMeta;
use super::slot_syntax::is_slot_directive;

pub(super) fn collect_child_types<'a, 'template, I>(
    children: I,
    source: &str,
    meta: ComponentSlotCheckMeta<'a, 'template>,
    skip_named_slot_templates: bool,
) -> std::vec::Vec<String>
where
    'template: 'a,
    I: IntoIterator<Item = &'a TemplateChildNode<'template>>,
{
    let mut output = std::vec::Vec::new();
    for child in children {
        collect_child_type(child, source, meta, skip_named_slot_templates, &mut output);
    }
    output
}

fn collect_child_type<'a, 'template>(
    child: &'a TemplateChildNode<'template>,
    source: &str,
    meta: ComponentSlotCheckMeta<'a, 'template>,
    skip_named_slot_templates: bool,
    output: &mut std::vec::Vec<String>,
) {
    match child {
        TemplateChildNode::Element(element) if element.tag == "template" => {
            let has_slot_directive = element.props.iter().any(is_slot_directive);
            if skip_named_slot_templates && has_slot_directive {
                return;
            }
            output.extend(collect_child_types(
                element.children.iter(),
                source,
                meta,
                skip_named_slot_templates,
            ));
        }
        TemplateChildNode::Element(element) => {
            if let Some(child_type) = element_slot_child_type(element, meta) {
                output.push(child_type);
            }
        }
        TemplateChildNode::Text(text) => {
            if !text.content.trim().is_empty() {
                output.push(String::from("string"));
            }
        }
        TemplateChildNode::Interpolation(_) | TemplateChildNode::CompoundExpression(_) => {
            output.push(String::from("string"));
        }
        TemplateChildNode::If(_)
        | TemplateChildNode::IfBranch(_)
        | TemplateChildNode::For(_)
        | TemplateChildNode::TextCall(_)
        | TemplateChildNode::Comment(_)
        | TemplateChildNode::Hoisted(_) => {}
    }
}

fn element_slot_child_type<'a, 'template>(
    element: &ElementNode<'template>,
    meta: ComponentSlotCheckMeta<'a, 'template>,
) -> Option<String> {
    if may_be_component_child(element)
        && !matches!(
            element.tag,
            "component" | "KeepAlive" | "Teleport" | "Transition" | "TransitionGroup" | "Suspense"
        )
        && let Some(component_ref) = resolved_component_binding_reference(
            meta.summary,
            meta.options,
            meta.syntactic_type_only_imported_names,
            element.tag,
        )
    {
        return Some(cstr!("typeof {component_ref}"));
    }

    match element.tag_type {
        ElementType::Component => None,
        ElementType::Element => Some(String::from(native_element_type(element.tag, element.ns))),
        ElementType::Slot | ElementType::Template => None,
    }
}

fn may_be_component_child(element: &ElementNode<'_>) -> bool {
    element.tag_type == ElementType::Component
        || element
            .tag
            .as_bytes()
            .first()
            .is_some_and(|first| first.is_ascii_uppercase())
        || element.tag.contains('-')
}

fn native_element_type(tag: &str, namespace: Namespace) -> &'static str {
    match namespace {
        Namespace::Svg => return "SVGElement",
        Namespace::MathMl => return "MathMLElement",
        Namespace::Html => {}
    }
    match tag {
        "a" => "HTMLAnchorElement",
        "area" => "HTMLAreaElement",
        "audio" => "HTMLAudioElement",
        "base" => "HTMLBaseElement",
        "blockquote" | "q" => "HTMLQuoteElement",
        "body" => "HTMLBodyElement",
        "br" => "HTMLBRElement",
        "button" => "HTMLButtonElement",
        "canvas" => "HTMLCanvasElement",
        "data" => "HTMLDataElement",
        "datalist" => "HTMLDataListElement",
        "del" | "ins" => "HTMLModElement",
        "details" => "HTMLDetailsElement",
        "dialog" => "HTMLDialogElement",
        "div" => "HTMLDivElement",
        "embed" => "HTMLEmbedElement",
        "fieldset" => "HTMLFieldSetElement",
        "form" => "HTMLFormElement",
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => "HTMLHeadingElement",
        "head" => "HTMLHeadElement",
        "hr" => "HTMLHRElement",
        "html" => "HTMLHtmlElement",
        "iframe" => "HTMLIFrameElement",
        "img" => "HTMLImageElement",
        "input" => "HTMLInputElement",
        "label" => "HTMLLabelElement",
        "legend" => "HTMLLegendElement",
        "li" => "HTMLLIElement",
        "link" => "HTMLLinkElement",
        "map" => "HTMLMapElement",
        "menu" | "ol" | "ul" => "HTMLUListElement",
        "meta" => "HTMLMetaElement",
        "meter" => "HTMLMeterElement",
        "object" => "HTMLObjectElement",
        "optgroup" => "HTMLOptGroupElement",
        "option" => "HTMLOptionElement",
        "output" => "HTMLOutputElement",
        "p" => "HTMLParagraphElement",
        "param" => "HTMLParamElement",
        "picture" => "HTMLPictureElement",
        "progress" => "HTMLProgressElement",
        "script" => "HTMLScriptElement",
        "select" => "HTMLSelectElement",
        "slot" => "HTMLSlotElement",
        "source" => "HTMLSourceElement",
        "span" => "HTMLSpanElement",
        "style" => "HTMLStyleElement",
        "table" => "HTMLTableElement",
        "tbody" | "tfoot" | "thead" => "HTMLTableSectionElement",
        "td" | "th" => "HTMLTableCellElement",
        "template" => "HTMLTemplateElement",
        "textarea" => "HTMLTextAreaElement",
        "time" => "HTMLTimeElement",
        "title" => "HTMLTitleElement",
        "tr" => "HTMLTableRowElement",
        "track" => "HTMLTrackElement",
        "video" => "HTMLVideoElement",
        _ => "HTMLElement",
    }
}
