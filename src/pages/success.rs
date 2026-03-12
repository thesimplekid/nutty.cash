use crate::components::bip353_box::Bip353Box;
use crate::components::button::{Button, ButtonFormat};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn SuccessPage() -> impl IntoView {
    let params = use_params_map();
    let address = Signal::derive(move || {
        params
            .get()
            .get("user")
            .map(|s| s.to_string())
            .unwrap_or_default()
    });

    let copied_address = RwSignal::new(false);
    let copied_link = RwSignal::new(false);

    let on_copy_address = move |_| {
        let adr = address.get();
        let window = window();
        let _ = window.navigator().clipboard().write_text(&adr);
        copied_address.set(true);
        set_timeout(
            move || copied_address.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    let on_share_link = move |_| {
        let window = window();
        let url = window.location().href().unwrap_or_default();
        let _ = window.navigator().clipboard().write_text(&url);
        copied_link.set(true);
        set_timeout(
            move || copied_link.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    view! {
        <div class="max-w-2xl mx-auto py-20 px-6 flex flex-col items-center gap-12 relative z-10">
            <div class="flex flex-col items-center gap-4 text-center">
                <div class="text-6xl animate-bounce">"🎉"</div>
                <h2 class="text-5xl font-black text-text-primary tracking-tight">"It's Yours!"</h2>
                <p class="text-lg text-text-secondary font-medium">
                    "Your Bitcoin payment address is live and resolving via DNS."
                </p>
            </div>

            <Bip353Box address=address/>

            <div class="flex flex-col md:flex-row gap-4 w-full justify-center">
                <Button class="px-8 py-4".to_string() on_click=Callback::new(on_copy_address)>
                    {move || if copied_address.get() { "Copied!" } else { "Copy Address" }}
                </Button>
                <Button
                    format=ButtonFormat::Secondary
                    class="px-8 py-4".to_string()
                    on_click=Callback::new(on_share_link)
                >
                    {move || if copied_link.get() { "Copied!" } else { "Share Link" }}
                </Button>
            </div>

            <leptos_router::components::A href="/new" attr:class="text-text-primary font-bold hover:underline mt-4">
                "Create Another"
            </leptos_router::components::A>
        </div>
    }
}
