use crate::components::button::{Button, ButtonFormat};
use crate::types::AppConfig;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    let config = use_context::<Resource<AppConfig>>().expect("AppConfig resource not found");
    let default_domain = move || {
        config
            .get()
            .map(|c| c.default_domain)
            .unwrap_or_else(|| "nutty.cash".to_string())
    };

    view! {
        <div class="flex flex-col items-center gap-12 max-w-4xl mx-auto py-20 px-6 relative z-10">
            <div class="flex flex-col items-center gap-4 text-center">
                <h1 class="text-5xl md:text-7xl font-black text-text-primary tracking-tighter">
                    "Bitcoin Payments" <br/>
                    <span class="text-text-secondary">"Human-Friendly."</span>
                </h1>
                <p class="text-xl text-text-secondary font-medium max-w-2xl">
                    "Ditch the long, confusing addresses. Claim your unique @"
                    <Suspense fallback=move || "nutty.cash".into_view()>
                        {default_domain}
                    </Suspense>
                    " handle and receive Bitcoin with ease. Secure, fast, and finally easy to read."
                </p>
            </div>

            <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
                <A href="/new">
                    <Button class="w-full md:w-auto px-10 py-5 text-xl".to_string()>
                        "Get Your Human Address"
                    </Button>
                </A>
                <A href="/search">
                    <Button format=ButtonFormat::Secondary class="w-full md:w-auto px-10 py-5 text-xl".to_string()>
                        "Look Up Address"
                    </Button>
                </A>
            </div>
        </div>
    }
}
