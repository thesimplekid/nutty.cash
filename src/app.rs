use crate::components::header::Header;
use crate::pages::home::HomePage;
use crate::pages::new_paycode::NewPayCodePage;
use crate::pages::success::SuccessPage;
use crate::api::get_app_config;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::pages::search::SearchPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
                <link href="https://fonts.googleapis.com/css2?family=Urbanist:ital,wght@0,100..900;1,100..900&display=swap" rel="stylesheet"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let config = Resource::new(|| (), |_| async move { get_app_config().await.unwrap_or_default() });
    provide_context(config);

    let pkg_name = env!("CARGO_PKG_NAME");
    let css_href = format!("/pkg/{}.css", pkg_name);

    let title = RwSignal::new("Nutty | Bitcoin Payments Human-Friendly".to_string());
    Effect::new(move |_| {
        if let Some(c) = config.get() {
            title.set(format!("{} | Bitcoin Payments Human-Friendly", c.app_name));
        }
    });

    view! {
        <Stylesheet id="leptos" href=css_href/>
        <Title text=move || title.get()/>

        <Router>
            <div class="page-wrapper font-urbanist">
                <Header/>
                <main class="flex-grow z-10 relative">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route path=StaticSegment("new") view=NewPayCodePage/>
                        <Route path=StaticSegment("search") view=SearchPage/>
                        <Route path=leptos_router::ParamSegment("user") view=SuccessPage/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
