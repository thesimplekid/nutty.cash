use leptos::prelude::*;

#[component]
pub fn Input(
    #[prop(into, optional)] label: Signal<String>,
    #[prop(into, optional)] description: Signal<String>,
    #[prop(into, optional)] placeholder: Signal<String>,
    #[prop(into, optional)] value: RwSignal<String>,
    #[prop(optional)] error: bool,
    #[prop(into, optional)] append: Signal<String>,
    #[prop(optional)] hidden: bool,
) -> impl IntoView {
    view! {
        <div class=format!("flex flex-col gap-2 w-full {}", if hidden { "hidden" } else { "" })>
            <label class="text-text-primary font-bold text-sm uppercase tracking-wider">{label}</label>
            <div class="relative flex items-center">
                <input
                    type="text"
                    class=format!(
                        "w-full p-4 bg-white dark:bg-bg-secondary border-2 rounded-xl text-text-primary font-medium focus:outline-none focus:border-text-primary transition-colors {}",
                        if error { "border-red-500" } else { "border-border-color" }
                    )
                    placeholder=placeholder
                    prop:value=value
                    on:input=move |ev| value.set(event_target_value(&ev))
                />
                <Show when=move || !append.get().is_empty() fallback=|| ()>
                    <span class="absolute right-4 text-text-secondary font-bold">{append}</span>
                </Show>
            </div>
            <Show when=move || !description.get().is_empty() fallback=|| ()>
                <p class=format!("text-xs font-medium {}", if error { "text-red-500" } else { "text-text-secondary" })>
                    {description}
                </p>
            </Show>
        </div>
    }
}
