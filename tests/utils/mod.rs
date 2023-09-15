use leptos::*;

#[track_caller]
pub fn check_str(component: impl IntoView, contains: &str) {
    let component_str = component.into_view().render_to_string();
    assert!(
        component_str.contains(contains),
        "expected \"{contains}\" to be found in the component render.\n\
        Found:\n\
        {component_str}"
    )
}
