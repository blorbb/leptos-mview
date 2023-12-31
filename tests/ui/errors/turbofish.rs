use leptos::*;
use leptos_mview::mview;

fn main() {
    use core::marker::PhantomData;
    // copied from https://github.com/leptos-rs/leptos/pull/1636
    #[component]
    pub fn GenericComponent<S>(ty: PhantomData<S>) -> impl IntoView {
        let _ty = ty;
        std::any::type_name::<S>()
    }

    mview! {
        GenericComponent::<String> ty={PhantomData};
    };
}
