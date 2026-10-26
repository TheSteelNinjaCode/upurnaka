// rahti-ui: attrs
// Installed from https://rahtiui.tsnc.tech/cli by cargo-rahti-ui 0.0.3.
//
// This is source, not a generated file — it is the application's to read and to
// change. `cargo rahti-ui update` re-fetches it, and leaves it alone once it has
// been edited; `--force` overwrites either way.

//! What a call site can say about a component's attributes.
//!
//! Every shadcn/ui component ends in `{...props}`: the caller's `type`,
//! `disabled`, `aria-*`, `data-*` and `onClick` land on the rendered element
//! without the component naming any of them. Rahti spells that as an explicit
//! prop — a component that forwards attributes declares `attrs: Attrs`, and
//! the caller passes it by name — because an unknown prop has to stay a
//! compile error rather than quietly becoming an HTML attribute nobody meant
//! to write.
//!
//! ```ignore
//! <Button attrs=@{Attrs::new().set("aria-label", "Sign up").unset("data-size")}>
//!     "Sign up"
//! </Button>
//! ```
//!
//! The set is then spread onto the element the component renders, with
//! `@{..attrs}`, and `html!` builds the open tag: names validated, values
//! escaped, and a later attribute replacing an earlier one in place rather
//! than being written beside it.

/// UI builder attributes: escaped server data plus typed, precompiled bindings.
/// Framework data sets can be converted with `Attrs::from(rahti_attrs)`.
/// Client code is supplied by `ui_expr!` or `ui_handler!`, never strings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Attrs(Vec<(String, Option<String>, bool)>);

impl Attrs {
    pub fn new() -> Self {
        Self::default()
    }
    fn put(mut self, name: String, value: Option<String>, bound: bool) -> Self {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.0 == name) {
            *entry = (name, value, bound);
        } else {
            self.0.push((name, value, bound));
        }
        self
    }
    pub fn set(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.put(name.into(), Some(value.into()), false)
    }
    pub fn unset(self, name: impl Into<String>) -> Self {
        self.put(name.into(), None, false)
    }
    pub fn bind(self, name: &str, expression: ClientBinding) -> Self {
        assert!(
            expression.name == name || (is_event_name(&expression.name) && is_event_name(name)),
            "rahti-ui: compile a binding for its actual attribute name"
        );
        self.bind_source(name, &expression.source)
    }
    fn bind_source(self, name: &str, source: &str) -> Self {
        self.put(name.to_string(), Some(format!("{{{source}}}")), true)
    }
    pub fn merge(self, other: &Self) -> Self {
        other.0.iter().fold(self, |attrs, (name, value, bound)| {
            attrs.put(name.clone(), value.clone(), *bound)
        })
    }
    pub fn only(&self, keep: impl Fn(&str) -> bool) -> Self {
        Self(
            self.0
                .iter()
                .filter(|entry| keep(&entry.0))
                .cloned()
                .collect(),
        )
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.0
            .iter()
            .map(|entry| (entry.0.as_str(), entry.1.as_deref()))
    }
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|entry| entry.0.as_str())
    }
    pub fn get(&self, name: &str) -> Option<Option<&str>> {
        self.0
            .iter()
            .find(|entry| entry.0 == name)
            .map(|entry| entry.1.as_deref())
    }
    pub fn is_bound(&self, name: &str) -> bool {
        self.0.iter().any(|entry| entry.0 == name && entry.2)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn data(&self) -> rahti::Attrs {
        self.0.iter().filter(|entry| !entry.2).fold(
            rahti::Attrs::new(),
            |attrs, (name, value, _)| match value {
                Some(value) => attrs.set(name, value),
                None => attrs.unset(name),
            },
        )
    }
}
impl From<rahti::Attrs> for Attrs {
    fn from(value: rahti::Attrs) -> Self {
        value
            .iter()
            .fold(Self::new(), |attrs, (name, value)| match value {
                Some(value) => attrs.set(name, value),
                None => attrs.unset(name),
            })
    }
}
impl<N: Into<String>, V: Into<String>, const K: usize> From<[(N, V); K]> for Attrs {
    fn from(values: [(N, V); K]) -> Self {
        values
            .into_iter()
            .fold(Self::new(), |attrs, (name, value)| attrs.set(name, value))
    }
}
impl rahti::Attributes for Attrs {
    fn write_attributes(&self, list: &mut rahti::AttrList) {
        for (name, value, bound) in &self.0 {
            match value {
                None => list.unset(name),
                Some(value) if *bound => {
                    let source = &value[1..value.len() - 1];
                    list.set_source(name, if is_event_name(name) { source } else { value });
                }
                Some(value) => list.set(name, value),
            }
        }
    }
}

/// Client code compiled by `html!`, with the attribute context it was compiled for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientBinding {
    name: String,
    source: String,
}
impl ClientBinding {
    /// Forward source supplied by Rahti's compiler-owned `Bindings` parameter.
    pub(crate) fn forwarded(name: &str, source: &'static str) -> Self {
        Self {
            name: name.to_string(),
            source: source.to_string(),
        }
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Extract the exact registry key produced by `html!`, once per macro site.
pub(crate) fn compiled_source(html: rahti::Html, name: &str) -> ClientBinding {
    let html = html.into_string();
    let attr = if is_event_name(name) {
        format!(" pp-{name}=\"")
    } else {
        format!(" {name}=\"")
    };
    let start = html.find(&attr).expect("compiled binding attribute") + attr.len();
    let source = html[start..].split('"').next().unwrap();
    let source = source
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    ClientBinding {
        name: name.to_string(),
        source: source
            .strip_prefix('{')
            .and_then(|source| source.strip_suffix('}'))
            .unwrap_or(&source)
            .trim()
            .to_string(),
    }
}

macro_rules! ui_expr {
    ($($name:ident)-* = $expression:tt) => {{
        static SOURCE: std::sync::OnceLock<$crate::components::rahti_ui::attrs::ClientBinding> = std::sync::OnceLock::new();
        SOURCE.get_or_init(|| $crate::components::rahti_ui::attrs::compiled_source(
            $crate::rahti::html! { <span $($name)-* = $expression></span> }, &stringify!($($name)-*).replace(' ', ""))).clone()
    }};
}
macro_rules! ui_handler {
    ($expression:tt) => {{
        static SOURCE: std::sync::OnceLock<$crate::components::rahti_ui::attrs::ClientBinding> =
            std::sync::OnceLock::new();
        SOURCE
            .get_or_init(|| {
                $crate::components::rahti_ui::attrs::compiled_source(
                    $crate::rahti::html! { <span onclick=$expression></span> },
                    "onclick",
                )
            })
            .clone()
    }};
}
pub(crate) use {ui_expr, ui_handler};

/// Caller-owned client props supplied by `html!` on a component tag.
/// Their compiled source is forwarded without evaluating or rebuilding it.
pub use rahti::Bindings;

/// Does this attribute value carry a PulsePoint expression?
///
/// Used only to decide whether legacy builder markup needs a mounted scope.
/// Binding code itself is compiled by `html!` and stored in `ClientBinding`;
/// this predicate never parses or compiles JavaScript.
pub fn is_binding(value: &str) -> bool {
    match value.find('{') {
        Some(open) => value[open + 1..].contains('}'),
        None => false,
    }
}

/// Caller bindings on a compound control's root, plus compiled reads for
/// the inner control. Root attributes retain the caller's names and typed
/// values; the inner element reads them through `pp.props`.
pub struct Lifted {
    /// Attributes for the component's root element.
    pub root: Attrs,
    /// Attributes for the element the bindings were meant for.
    pub inner: Attrs,
}

/// Split `bindings` into [`Lifted`]'s two halves.
pub fn lift_bindings(bindings: Bindings) -> Lifted {
    let mut attrs = Attrs::new();
    for (name, source) in bindings.iter() {
        attrs = attrs.bind(name, ClientBinding::forwarded(name, source));
    }
    lift_attrs(&attrs)
}

pub fn lift_attrs(attrs: &Attrs) -> Lifted {
    let root = attrs.only(|name| attrs.is_bound(name));
    let mut inner = Attrs::new();
    for name in attrs.names().filter(|name| attrs.is_bound(name)) {
        if is_event_name(name) || name == "pp-ref" {
            continue;
        }
        inner = inner.bind(name, read_prop(name));
    }
    Lifted { root, inner }
}

fn read_prop(name: &str) -> ClientBinding {
    match name {
        "checked" => ui_expr!(checked = { pp.props.checked }),
        "value" => ui_expr!(value = { pp.props.value }),
        "disabled" => ui_expr!(disabled = { pp.props.disabled }),
        "required" => ui_expr!(required = { pp.props.required }),
        "open" => ui_expr!(open = { pp.props.open }),
        "hidden" => ui_expr!(hidden = { pp.props.hidden }),
        "class" => ui_expr!(class = { pp.props.class }),
        "style" => ui_expr!(style = { pp.props.style }),
        "id" => ui_expr!(id = { pp.props.id }),
        "name" => ui_expr!(name = { pp.props.name }),
        "title" => ui_expr!(title = { pp.props.title }),
        "min" => ui_expr!(min = { pp.props.min }),
        "max" => ui_expr!(max = { pp.props.max }),
        "step" => ui_expr!(step = { pp.props.step }),
        "aria-label" => ui_expr!(aria - label = { pp.props.ariaLabel }),
        "aria-labelledby" => ui_expr!(aria - labelledby = { pp.props.ariaLabelledby }),
        "aria-invalid" => ui_expr!(aria - invalid = { pp.props.ariaInvalid }),
        "aria-pressed" => ui_expr!(aria - pressed = { pp.props.ariaPressed }),
        "aria-checked" => ui_expr!(aria - checked = { pp.props.ariaChecked }),
        "aria-current" => ui_expr!(aria - current = { pp.props.ariaCurrent }),
        "data-state" => ui_expr!(data - state = { pp.props.dataState }),
        "data-disabled" => ui_expr!(data - disabled = { pp.props.dataDisabled }),
        "data-invalid" => ui_expr!(data - invalid = { pp.props.dataInvalid }),
        "data-active" => ui_expr!(data - active = { pp.props.dataActive }),
        "data-open" => ui_expr!(data - open = { pp.props.dataOpen }),
        "pp-ref" => ui_expr!(pp-ref={pp.props.ppRef}),
        "pp-style" => ui_expr!(pp - style = { pp.props.ppStyle }),
        "type" => ui_expr!(type={pp.props.type}),
        "placeholder" => ui_expr!(placeholder = { pp.props.placeholder }),
        "autofocus" => ui_expr!(autofocus = { pp.props.autofocus }),
        "multiple" => ui_expr!(multiple = { pp.props.multiple }),
        "readonly" => ui_expr!(readonly = { pp.props.readonly }),
        _ => panic!(
            "rahti-ui: `{name}` cannot be forwarded to an inner control; bind it on the root or add its compiled prop mapping"
        ),
    }
}

/// A native event attribute: `on` and lowercase letters, the runtime's own
/// test. `on-pick` is a custom prop, not an event.
fn is_event_name(name: &str) -> bool {
    name.len() > 2 && name.starts_with("on") && name[2..].bytes().all(|b| b.is_ascii_lowercase())
}

/// The runtime's attribute-to-prop rename: `aria-label-prop` -> `ariaLabelProp`.
fn camel_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '-' {
            upper = true;
        } else if upper && c.is_ascii_lowercase() {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            if upper {
                out.push('-');
            }
            out.push(c);
            upper = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Attrs, Bindings, camel_case, is_binding, lift_bindings};

    #[test]
    fn compiled_builder_sources_match_the_registry() {
        use rahti::client::{ClientCode, ClientKind};
        let handler = super::ui_handler!({ save() });
        let expression = super::ui_expr!(title = { count + 1 });
        for (binding, kind) in [
            (handler, ClientKind::Handler),
            (expression, ClientKind::Expression),
        ] {
            assert!(
                rahti::inventory::iter::<ClientCode>
                    .into_iter()
                    .any(|entry| entry.kind == kind && entry.key == binding.source()),
                "{}",
                binding.source()
            );
        }
    }

    #[test]
    fn events_go_on_the_root_and_values_are_passed_down_as_props() {
        let lifted = lift_bindings(Bindings::__from_html(&[
            ("checked", "bold"),
            ("onchange", "setBold(target.checked)"),
            ("aria-label", "label"),
        ]));

        assert_eq!(
            lifted.root.get("onchange"),
            Some(Some("{setBold(target.checked)}"))
        );
        assert_eq!(lifted.root.get("checked"), Some(Some("{bold}")));
        assert_eq!(lifted.root.get("aria-label"), Some(Some("{label}")));
        assert_eq!(lifted.root.get("checked"), Some(Some("{bold}")));

        assert_eq!(
            lifted.inner.get("checked"),
            Some(Some("{pp.props.checked}"))
        );
        assert_eq!(
            lifted.inner.get("aria-label"),
            Some(Some("{pp.props.ariaLabel}"))
        );
        assert_eq!(lifted.inner.get("onchange"), None);

        // Every half is a live binding, never text whose braces get escaped.
        for name in ["onchange", "checked", "aria-label"] {
            assert!(lifted.root.is_bound(name), "{name}");
        }
        assert!(lifted.inner.is_bound("checked"));
    }

    /// The runtime only uppercases a letter after a dash; anything else keeps
    /// its dash.
    #[test]
    fn the_prop_name_is_the_runtimes_camel_case() {
        assert_eq!(camel_case("checked"), "checked");
        assert_eq!(camel_case("aria-label"), "ariaLabel");
        assert_eq!(camel_case("x-1-prop"), "x-1Prop");
    }

    #[test]
    fn a_binding_is_recognized_by_its_braces() {
        assert!(is_binding("{save()}"));
        assert!(is_binding("busy: {busy}"));
        assert!(!is_binding("submit"));
        assert!(!is_binding("inline-flex items-center"));
        // A brace with nothing closing it is not an expression.
        assert!(!is_binding("{unclosed"));
    }

    /// Data attributes retain framework last-wins and removal semantics.
    #[test]
    fn attrs_preserves_data_and_removals() {
        let attrs = Attrs::new().set("type", "submit").unset("data-size");

        assert_eq!(attrs.get("type"), Some(Some("submit")));
        assert_eq!(attrs.get("data-size"), Some(None));
        assert_eq!(attrs.get("id"), None);
    }
}
