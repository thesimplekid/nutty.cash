use crate::api::lookup_bip353;
use crate::components::button::Button;
use crate::components::input::Input;
use crate::types::AppConfig;
use leptos::either::Either;
use leptos::prelude::*;

#[component]
fn DataBox(label: &'static str, value: String, emoji: &'static str) -> impl IntoView {
    let copied = RwSignal::new(false);
    let value_copy = value.clone();
    let on_copy = move |_| {
        let window = window();
        let _ = window.navigator().clipboard().write_text(&value_copy);
        copied.set(true);
        set_timeout(move || copied.set(false), std::time::Duration::from_secs(2));
    };

    view! {
        <div class="flex flex-col gap-2 p-4 bg-white/10 dark:bg-bg-secondary/40 rounded-2xl border border-border-color shadow-sm relative group">
            <div class="flex justify-between items-center">
                <span class="text-xs font-bold uppercase text-text-secondary flex items-center gap-1">
                    <span>{emoji}</span> {label}
                </span>
                <button
                    on:click=on_copy
                    class="p-1.5 hover:bg-black/5 dark:hover:bg-white/5 rounded-md transition-colors text-xs font-bold text-text-secondary"
                >
                    {move || if copied.get() { "Copied!" } else { "Copy" }}
                </button>
            </div>
            <div class="font-mono break-all text-sm text-text-primary max-h-32 overflow-y-auto">
                {value}
            </div>
        </div>
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let config = use_context::<Resource<AppConfig>>().expect("AppConfig resource not found");
    let default_domain = move || config.get().map(|c| c.default_domain).unwrap_or_else(|| "nutty.cash".to_string());

    let query = RwSignal::new(String::new());
    let lookup_action = Action::new(|address: &String| {
        let address = address.clone();
        async move { lookup_bip353(address).await }
    });

    let result = lookup_action.value();

    view! {
        <Suspense fallback=move || view! { <div class="max-w-2xl mx-auto py-20 px-6">"Loading..."</div> }>
            <div class="max-w-2xl mx-auto py-20 px-6 flex flex-col gap-8 items-center text-center relative z-10">
                <div class="flex flex-col gap-4">
                    <h2 class="text-4xl font-black text-text-primary tracking-tight">"Look Up Address"</h2>
                    <p class="text-text-secondary font-medium">"Search for a BIP-353 human address to see payment details."</p>
                </div>

                <div class="w-full flex flex-col gap-6 bg-white/20 dark:bg-bg-secondary/20 p-8 rounded-3xl border-2 border-border-color shadow-xl backdrop-blur-md">
                    <Input
                        label="Lightning Address"
                        placeholder=Signal::derive(move || format!("user@{}", default_domain()))
                        value=query
                    />

                    <Button
                        class="w-full py-4 text-xl".to_string()
                        on_click=Callback::new(move |_| {
                            lookup_action.dispatch(query.get());
                        })
                        disabled=Signal::derive(move || lookup_action.pending().get())
                    >
                        {move || if lookup_action.pending().get() { "Searching..." } else { "Lookup Details" }}
                    </Button>
                </div>

                {move || {
                    result.get().map(|res| {
                        match res {
                            Ok(res) => {
                                let uri = res.uri.clone();
                                let copied = RwSignal::new(false);
                                let on_copy = move |_| {
                                    let window = window();
                                    let _ = window.navigator().clipboard().write_text(&uri);
                                    copied.set(true);
                                    set_timeout(move || copied.set(false), std::time::Duration::from_secs(2));
                                };

                                Either::Left(view! {
                                    <div class="w-full p-6 bg-white/20 dark:bg-bg-secondary/20 rounded-3xl border-2 border-border-color shadow-xl backdrop-blur-md flex flex-col gap-6 text-left animate-in fade-in slide-in-from-bottom-4 duration-500">
                                        <div class="flex justify-between items-center">
                                            <h3 class="text-xl font-bold text-text-primary">"Lookup Result"</h3>
                                            <span class="px-2 py-1 bg-green-500/10 text-green-500 text-[10px] font-bold uppercase rounded-md border border-green-500/20">"Resolved"</span>
                                        </div>
                                        
                                        <div class="flex flex-col gap-1">
                                            <span class="text-xs font-bold uppercase text-text-secondary">"Identifier"</span>
                                            <span class="font-mono break-all text-text-primary text-lg">{res.address}</span>
                                        </div>

                                        <div class="grid grid-cols-1 gap-4">
                                            {res.lno.map(|v| view! { <DataBox label="BOLT 12 Offer" value=v emoji="⚡" /> })}
                                            {res.sp.map(|v| view! { <DataBox label="Silent Payment" value=v emoji="🔗" /> })}
                                            {res.creq.map(|v| view! { <DataBox label="Cashu Request" value=v emoji="🪙" /> })}
                                            {res.onchain_address.map(|v| view! { <DataBox label="On-Chain Address" value=v emoji="⛓️" /> })}
                                        </div>

                                        <div class="flex flex-col gap-2">
                                            <span class="text-xs font-bold uppercase text-text-secondary">"BIP-321 Unified URI"</span>
                                            <div class="relative group">
                                                <div class="font-mono break-all text-[10px] bg-black/5 dark:bg-white/5 p-4 rounded-xl border border-border-color max-h-32 overflow-y-auto">
                                                    {res.uri.clone()}
                                                </div>
                                                <button 
                                                    on:click=on_copy
                                                    class="absolute top-2 right-2 p-2 bg-white dark:bg-bg-secondary border border-border-color rounded-lg shadow-sm hover:scale-105 transition-transform"
                                                >
                                                    {move || if copied.get() { "✅" } else { "📋" }}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                })
                            }
                            Err(e) => {
                                Either::Right(view! {
                                    <div class="w-full p-4 bg-red-500/10 border-2 border-red-500/20 rounded-2xl text-red-500 font-medium animate-in fade-in slide-in-from-bottom-2">
                                        {e.to_string()}
                                    </div>
                                })
                            }
                        }
                    })
                }}
            </div>
        </Suspense>
    }
}
