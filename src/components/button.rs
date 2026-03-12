use leptos::ev;
use leptos::prelude::*;

#[derive(Default, Clone, PartialEq)]
pub enum ButtonFormat {
    #[default]
    Primary,
    Secondary,
    Outline,
}

#[component]
pub fn Button(
    #[prop(into, optional)] on_click: Option<Callback<ev::MouseEvent>>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] active: Signal<bool>,
    #[prop(optional)] format: ButtonFormat,
    #[prop(into, optional)] class: String,
    children: Children,
) -> impl IntoView {
    let base_classes = "px-6 py-3 rounded-xl font-bold transition-all flex items-center gap-2 justify-center disabled:opacity-50 disabled:cursor-not-allowed";

    let format_classes = move || {
        match format {
        ButtonFormat::Primary => {
            "bg-black dark:bg-white text-white dark:text-black hover:opacity-90 shadow-lg"
                .to_string()
        }
        ButtonFormat::Secondary => {
            "bg-white dark:bg-bg-secondary text-text-primary hover:bg-gray-50 dark:hover:bg-gray-800 border-2 border-border-color"
                .to_string()
        }
        ButtonFormat::Outline => {
            if active.get() {
                "bg-black dark:bg-white text-white dark:text-black".to_string()
            } else {
                "bg-transparent text-text-primary border-2 border-border-color hover:bg-bg-secondary"
                    .to_string()
            }
        }
    }
    };

    view! {
        <button
            class=move || format!("{base_classes} {} {class}", format_classes())
            disabled=move || disabled.get()
            on:click=move |ev| if let Some(cb) = on_click { cb.run(ev) }
        >
            {children()}
        </button>
    }
}
