use crate::types::AppConfig;
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Header() -> impl IntoView {
    let config = use_context::<Resource<AppConfig>>().expect("AppConfig resource not found");

    view! {
        <header class="px-4 py-4 md:px-6 md:py-6 flex justify-between items-center max-w-6xl mx-auto w-full z-20 relative">
            <A href="/" attr:class="flex items-center gap-2 flex-shrink-0">
                <Suspense fallback=move || view! {
                    <div class="w-10 h-10 bg-black dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-black font-bold text-xl">
                        "N"
                    </div>
                    <span class="text-text-primary font-bold text-2xl hidden md:inline">"Nutty"</span>
                }>
                    {move || {
                        let config_val = config.get();
                        let app_name = config_val.as_ref().map(|c| c.app_name.clone()).unwrap_or_else(|| "Nutty".to_string());
                        let app_initial = app_name.chars().next().unwrap_or('N').to_string();
                        view! {
                            <div class="w-10 h-10 bg-black dark:bg-white rounded-lg flex items-center justify-center text-white dark:text-black font-bold text-xl">
                                {app_initial}
                            </div>
                            <span class="text-text-primary font-bold text-2xl hidden md:inline">{app_name}</span>
                        }
                    }}
                </Suspense>
            </A>
            <nav class="flex gap-4 md:gap-6 items-center flex-shrink-0">
                <A href="/new" attr:class="text-text-primary font-medium hover:underline text-sm md:text-base whitespace-nowrap">"Get Code"</A>
                <A href="/search" attr:class="text-text-primary font-medium hover:underline text-sm md:text-base whitespace-nowrap">"Look Up"</A>
            </nav>
        </header>
    }
}
