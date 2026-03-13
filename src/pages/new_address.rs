use crate::api::CreateAddressServer;
use crate::components::button::{Button, ButtonFormat};
use crate::components::input::Input;
use crate::types::{AppConfig, CreateAddressRequest};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NewAddressPage() -> impl IntoView {
    let config = use_context::<Resource<AppConfig>>().expect("AppConfig resource not found");
    let navigate = use_navigate();
    let user_name = RwSignal::new(String::new());

    let default_domain = move || config.get().map(|c| c.default_domain).unwrap_or_else(|| "nutty.cash".to_string());
    let domain = RwSignal::new("nutty.cash".to_string());

    Effect::new(move |_| {
        domain.set(default_domain());
    });

    let lno = RwSignal::new(String::new());
    let sp = RwSignal::new(String::new());
    let creq = RwSignal::new(String::new());
    let cashu_token = RwSignal::new(String::new());

    let free_name = RwSignal::new(false);
    let error_msg = RwSignal::new(None::<String>);
    let is_busy = RwSignal::new(false);

    // Payment required state: (amount, suggested_username, accepted_mints)
    let payment_info = RwSignal::new(None::<(u64, String, Vec<String>)>);

    let create_action = ServerAction::<CreateAddressServer>::new();

    let navigate_submit = navigate.clone();
    let on_submit = move |_| {
        let navigate = navigate_submit.clone();
        is_busy.set(true);
        error_msg.set(None);

        let req = CreateAddressRequest {
            user_name: if free_name.get() {
                payment_info.get().map(|p| p.1).or(None)
            } else {
                Some(user_name.get())
            },
            domain: domain.get(),
            lno: if lno.get().is_empty() {
                None
            } else {
                Some(lno.get())
            },
            sp: if sp.get().is_empty() {
                None
            } else {
                Some(sp.get())
            },
            creq: if creq.get().is_empty() {
                None
            } else {
                Some(creq.get())
            },
        };

        if let Err(e) = req.validate() {
            error_msg.set(Some(e));
            is_busy.set(false);
            return;
        }

        // If we have a token, we need to send it via header.
        // Since Leptos doesn't easily allow this, we'll use fetch if token is present.
        let token = cashu_token.get();
        if !token.is_empty() {
            spawn_local(async move {
                let url = "/api/v1/address";
                let client = gloo_net::http::Request::post(url)
                    .header("X-Cashu", &token)
                    .json(&req);

                match client {
                    Ok(req_builder) => match req_builder.send().await {
                        Ok(resp) => {
                            if resp.ok() {
                                match resp.json::<serde_json::Value>().await {
                                    Ok(data) => {
                                        let user = data["user_name"].as_str().unwrap_or("");
                                        let dom = data["domain"].as_str().unwrap_or("");
                                        if !user.is_empty() && !dom.is_empty() {
                                            navigate(
                                                &format!("/{}@{}", user, dom),
                                                Default::default(),
                                            );
                                        } else {
                                            error_msg.set(Some("Unexpected response from server".into()));
                                        }
                                    }
                                    Err(e) => error_msg.set(Some(format!("Failed to parse response: {}", e))),
                                }
                            } else {
                                let status = resp.status();
                                let body = resp.json::<serde_json::Value>().await
                                    .ok()
                                    .and_then(|v| v["error"].as_str().map(|s| s.to_string()));
                                error_msg.set(Some(body.unwrap_or_else(|| format!("Error: {}", status))));
                            }
                        }
                        Err(e) => error_msg.set(Some(e.to_string())),
                    },
                    Err(e) => error_msg.set(Some(e.to_string())),
                }
                is_busy.set(false);
            });
        } else {
            create_action.dispatch(CreateAddressServer { req });
        }
    };

    let navigate_effect = navigate.clone();
    Effect::new(move |_| {
        if let Some(res) = create_action.value().get() {
            is_busy.set(false);
            match res {
                Ok(pc) => {
                    navigate_effect(
                        &format!("/{}@{}", pc.user_name, pc.domain),
                        Default::default(),
                    );
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if let Some(idx) = err_str.find("PAYMENT_REQUIRED:") {
                        let json_str = &err_str[idx + "PAYMENT_REQUIRED:".len()..];
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                            let amount = data["amount"].as_u64().unwrap_or(0);
                            let suggested = data["user_name"].as_str().unwrap_or("").to_string();
                            let mints = data["accepted_mints"]
                                .as_array()
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            payment_info.set(Some((amount, suggested, mints)));
                        }
                    } else {
                        error_msg.set(Some(err_str));
                    }
                }
            }
        }
    });

    view! {
        <div class="max-w-xl mx-auto py-12 px-6 flex flex-col gap-10 relative z-10">
            <div class="flex flex-col gap-4 text-center">
                <h2 class="text-4xl font-black text-text-primary tracking-tight">"Get Your Human Bitcoin Address"</h2>
                <Show when=move || payment_info.get().is_none() fallback=|| ()>
                    <div class="flex gap-2 justify-center">
                        <Button
                            format=ButtonFormat::Outline
                            active=Signal::derive(move || !free_name.get())
                            on_click=Callback::new(move |_| free_name.set(false))
                        >
                            "Choose a Name"
                        </Button>
                        <Button
                            format=ButtonFormat::Outline
                            active=Signal::derive(move || free_name.get())
                            on_click=Callback::new(move |_| free_name.set(true))
                        >
                            "Random Name"
                        </Button>
                    </div>
                </Show>
            </div>

            <div class="flex flex-col gap-6 bg-white/20 dark:bg-bg-secondary/20 p-8 rounded-3xl border-2 border-border-color shadow-xl backdrop-blur-md">
                <Show when=move || payment_info.get().is_none() fallback=move || {
                    let (amount, suggested, mints) = payment_info.get().unwrap();
                    view! {
                        <div class="flex flex-col gap-6 p-8 bg-white/30 dark:bg-white/5 rounded-2xl border-2 border-text-primary/10 dark:border-white/10 backdrop-blur-md animate-in fade-in slide-in-from-bottom-4 shadow-2xl">
                            <div class="flex flex-col gap-2">
                                <h3 class="text-xl font-black text-text-primary text-center">"Payment Required"</h3>
                                <p class="text-sm text-center text-text-secondary">
                                    "The address " <span class="font-bold text-text-primary">{suggested}</span> " requires a one-time payment of "
                                    <span class="font-bold text-text-primary">{amount}</span> " sats."
                                </p>
                            </div>
                            <div class="flex flex-col gap-3">
                                <p class="font-bold text-xs uppercase tracking-wider text-text-secondary text-center">"Accepted Mints"</p>
                                <div class="max-h-48 overflow-y-auto px-2 flex flex-col gap-3">
                                    {mints.into_iter().map(|mint| view! {
                                        <div class="group relative flex flex-col p-4 bg-black/5 dark:bg-white/5 rounded-xl border border-border-color hover:border-text-primary/30 transition-all overflow-hidden shadow-sm hover:shadow-md">
                                            <div class="items-center gap-2 mb-2 flex">
                                                <div class="w-2 h-2 rounded-full bg-text-primary/20 group-hover:bg-text-primary transition-colors"></div>
                                                <span class="text-[10px] font-black uppercase tracking-widest text-text-secondary">"Cashu Mint"</span>
                                            </div>
                                            <span class="text-xs break-words font-medium text-text-primary leading-normal">
                                                {mint}
                                            </span>
                                        </div>
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                            <Input
                                label="Paste Cashu Token"
                                placeholder="cashuB..."
                                value=cashu_token
                                description="Paste the token from your wallet here."
                            />
                        </div>
                    }
                }>
                    <Show when=move || !free_name.get() fallback=|| ()>
                        <Input
                            label="Choose a User Name"
                            placeholder="satoshi"
                            value=user_name
                            append=Signal::derive(move || format!("@{}", domain.get()))
                        />
                    </Show>

                    <Input
                        label="Cashu Payment Request"
                        placeholder="creqA... or creqB..."
                        value=creq
                        description="Cashu payment request; creqA and creqB are both accepted"
                    />

                    <Input
                        label="BOLT 12 Offer"
                        placeholder="lno123..."
                        value=lno
                        description="Standard for Lightning payments"
                    />

                    <Input
                        label="Silent Payments address"
                        placeholder="sp123..."
                        value=sp
                        description="Private on-chain payments"
                    />
                </Show>

                {move || error_msg.get().map(|e| view! { <p class="text-red-600 text-sm font-bold text-center">{e}</p> })}

                <div class="flex flex-col md:flex-row gap-4 justify-end mt-4">
                    <Button
                        disabled=Signal::derive(move || is_busy.get())
                        on_click=Callback::new(on_submit)
                    >
                        {move || if is_busy.get() { "Processing..." } else if payment_info.get().is_some() { "Complete Payment" } else { "Create Human Bitcoin Address" }}
                    </Button>
                </div>
            </div>
        </div>
    }
}
