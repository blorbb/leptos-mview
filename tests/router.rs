use leptos::prelude::*;
use leptos_router::{components::{Route, Router, Routes}, location::RequestUrl, StaticSegment};
use leptos_mview::mview;

mod utils;
use utils::check_str;

#[test]
fn router() {
    #[component]
    fn RouterContext(children: ChildrenFn, path: &'static str) -> impl IntoView {
        // `Router` panicks if it is not provided with a `RequestUrl` context
        Owner::new().set();
        provide_context(RequestUrl::new(path));

        mview! {
            {children()}
        }
    }


    let router = || mview! {
        Router {
            main {
                Routes
                    fallback=[mview! { p { "not found" }}]
                {
                    Route
                        path={StaticSegment("")}
                        view=[mview! { p { "root route" } }];

                    Route
                        path={StaticSegment("route2")}
                        view=[mview! { p { "you are on /route2" } }];
                }
            }
        }
    };

    let router_context1 = mview! {
        RouterContext
            path="/"
        {{
            router()
        }}
    };

    let router_context2 = mview! {
        RouterContext
            path="/route2"
        {{
            router()
        }}
    };

    let router_context3 = mview! {
        RouterContext
            path="/does-not-exist"
        {{
            router()
        }}
    };

    check_str(router_context1, "<p>root route");
    check_str(router_context2, "<p>you are on /route2");
    check_str(router_context3, "<p>not found");
}
