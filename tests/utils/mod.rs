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
        Contains::All(a) => a.into_iter().for_each(|s| {
            assert!(
                component_str.contains(s),
                "expected all of {a:?} to be found in the component render.\n\
                did not find {s:?}\n\
                Found:\n\
                {component_str}"
            )
        }),
        Contains::Not(s) => {
            assert!(
                !component_str.contains(s),
                "expected \"{s}\" to not be found in the component render.\n\
                Found:\n\
                {component_str}"
            )
        }
        Contains::NoneOf(a) => a.into_iter().for_each(|s| {
            assert!(
                !component_str.contains(s),
                "expected none of {a:?} to be found in the component render.\n\
                found {s:?} in the component:\n\
                {component_str}"
            )
        }),
        Contains::AllOfNoneOf([a, n]) => {
            a.into_iter().for_each(|s| {
                assert!(
                    component_str.contains(s),
                    "expected all of {a:?} to be found in the component render.\n\
                    did not find {s:?}\n\
                    Found:\n\
                    {component_str}"
                );
            });
            n.into_iter().for_each(|s| {
                assert!(
                    !component_str.contains(s),
                    "expected none of {n:?} to be found in the component render.\n\
                    found {s:?} in the component:\n\
                    {component_str}"
                );
            });
        }
    };
}

pub enum Contains<'a> {
    Str(&'a str),
    All(&'a [&'a str]),
    Not(&'a str),
    NoneOf(&'a [&'a str]),
    AllOfNoneOf([&'a [&'a str]; 2]),
}

impl<'a> From<&'a str> for Contains<'a> {
    fn from(value: &'a str) -> Self { Self::Str(value) }
}

impl<'a> From<&'a [&'a str]> for Contains<'a> {
    fn from(value: &'a [&'a str]) -> Self { Self::All(value) }
}
