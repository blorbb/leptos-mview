use leptos::*;

#[track_caller]
pub fn check_str<'a>(component: impl IntoView, contains: impl Into<Contains<'a>>) {
    let component_str = component.into_view().render_to_string();
    match contains.into() {
        Contains::Str(s) => {
            assert!(
                component_str.contains(s),
                "expected \"{s}\" to be found in the component render.\n\
                Found:\n\
                {component_str}"
            )
        }
        Contains::Slice(a) => a.into_iter().for_each(|s| {
            assert!(
                component_str.contains(s),
                "expected all of {a:?} to be found in the component render.\n\
                did not find {s:?}
                Found:\n\
                {component_str}"
            )
        }),
    };
}

pub enum Contains<'a> {
    Str(&'a str),
    Slice(&'a [&'a str]),
}

impl<'a> From<&'a str> for Contains<'a> {
    fn from(value: &'a str) -> Self {
        Self::Str(value)
    }
}

impl<'a> From<&'a [&'a str]> for Contains<'a> {
    fn from(value: &'a [&'a str]) -> Self {
        Self::Slice(value)
    }
}
