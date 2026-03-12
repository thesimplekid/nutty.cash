use leptos::prelude::*;

#[component]
pub fn Bip353Box(#[prop(into)] address: Signal<String>) -> impl IntoView {
    view! {
        <div class="bg-black dark:bg-bg-secondary text-white dark:text-text-primary p-8 rounded-3xl shadow-2xl flex flex-col items-center gap-4 border-t-8 border-border-color">
            <span class="text-sm font-bold uppercase tracking-widest text-text-secondary">"Your BIP-353 Address"</span>
            <span class="text-2xl md:text-4xl font-black break-all text-center">{move || address.get()}</span>
        </div>
    }
}
