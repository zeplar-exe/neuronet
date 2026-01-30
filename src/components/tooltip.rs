use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct TooltipIndicatorProps {
    pub kind: TooltipKind,
    pub text: String,
    #[props(default = TooltipPosition::Top)]
    pub position: TooltipPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooltipKind {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TooltipPosition {
    Top,
    Right,
    Bottom,
    Left,
}

impl TooltipKind {
    fn class(&self) -> &'static str {
        match self {
            TooltipKind::Info => "info",
            TooltipKind::Warning => "warn",
            TooltipKind::Error => "error",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            TooltipKind::Info => "I",
            TooltipKind::Warning => "W",
            TooltipKind::Error => "E",
        }
    }

    fn aria_label(&self) -> &'static str {
        match self {
            TooltipKind::Info => "Info",
            TooltipKind::Warning => "Warning",
            TooltipKind::Error => "Error",
        }
    }
}

impl TooltipPosition {
    fn class(&self) -> &'static str {
        match self {
            TooltipPosition::Top => "top",
            TooltipPosition::Right => "right",
            TooltipPosition::Bottom => "bottom",
            TooltipPosition::Left => "left",
        }
    }
}

#[component]
pub fn TooltipIndicator(props: TooltipIndicatorProps) -> Element {
    let kind_cls = props.kind.class();
    let pos_cls = props.position.class();
    let aria = props.kind.aria_label();

    rsx! {
        div { class: "nn-tip {kind_cls} {pos_cls}", tabindex: 0, role: "button", aria_label: aria,
            span { class: "nn-tip-icon", {props.kind.icon()} }
            div { class: "nn-tip-bubble nn-tip-bubble--{kind_cls} nn-tip-bubble--{pos_cls}",
                span { class: "nn-tip-text", {props.text} }
            }
        }
    }
}
