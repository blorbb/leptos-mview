use leptos::{context::Provider, prelude::*};
use leptos_mview::mview;
use leptos_router::{
    components::{Route, Router, Routes},
    location::RequestUrl,
    path,
};

mod utils;
use utils::check_str;

#[test]
fn router() {
    #[component]
    fn RouterContext(children: ChildrenFn, path: &'static str) -> impl IntoView {
        // `Router` panicks if it is not provided with a `RequestUrl` context
        mview! {
            Provider value={RequestUrl::new(path)} (
                {children()}
            )
        }
    }

    let router = || {
        mview! {
            Router {
                main {
                    Routes
                        fallback=[mview! { p("not found")}]
                    (
                        Route
                            path={path!("")}
                            view=[mview! { p("root route") }];

                        Route
                            path={path!("route2")}
                            view=[mview! { p("you are on /route2") }];
                    )
                }
            }
        }
    };

    Owner::new().with(|| {
        let router_context1 = mview! {
            RouterContext path="/" (
                {router()}
            )
        };

        let router_context2 = mview! {
            RouterContext path="/route2" (
                {router()}
            )
        };

        let router_context3 = mview! {
            RouterContext path="/does-not-exist" (
                {router()}
            )
        };

        check_str(router_context1, "<p>root route");
        check_str(router_context2, "<p>you are on /route2");
        check_str(router_context3, "<p>not found");
    });
}
