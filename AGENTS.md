# AGENTS.md

## Project Overview
- `nutty` is a Rust 2021 Leptos application for human-friendly Bitcoin payment addresses and paycode registration.
- The crate builds both a hydrated WASM frontend and an SSR Axum server; feature flags matter for most commands.
- Core domains: BIP-353 lookup, BIP-321 URI generation, Cashu payment handling, and Cloudflare DNS record management.
- Main source roots are `src/`, `style/`, `public/`, plus deployment-oriented Nix files in `flake.nix` and `nix/`.
- There is no existing `AGENTS.md`, no `.cursorrules`, no `.cursor/rules/`, and no `.github/copilot-instructions.md` in this repo.

## Repository Layout
- `src/lib.rs` defines the shared crate modules and the `hydrate()` entrypoint behind the `hydrate` feature.
- `src/main.rs` starts the SSR app behind the `ssr` feature and wires Axum, Leptos routes, and app state.
- `src/app.rs` defines the shell HTML and top-level router.
- `src/api.rs` contains Leptos server functions used by the frontend.
- `src/server/` holds SSR-only infrastructure: API handlers, state bootstrapping, DB access, random-name generation, and Cloudflare integration.
- `src/pages/` contains Leptos page components; `src/components/` contains reusable UI primitives.
- `src/types.rs` defines the main shared request/response and persistence types.
- `style/main.css` is the hand-rolled utility stylesheet; there is no Tailwind config in this repo.
- `SKILL.md` is user-facing API documentation that is also served from HTTP routes.

## Toolchain And Environment
- Rust edition is `2021`; use the standard Rust toolchain unless the repo’s Nix environment is required locally.
- The project depends on Leptos `0.8`, Axum `0.8`, Tokio `1`, and optional SSR-only wallet/Cloudflare dependencies.
- For full SSR startup, environment variables are required; see `.env.example` and `README.md` for the minimum set.
- Important runtime variables include `CF_TOKEN`, `DOMAINS`, `NETWORK`, `ACCEPTED_MINTS`, `CDK_MNEMONIC`, `SITE_ADDR`, and optional pricing/payout variables.
- `dotenvy` is loaded in `AppState::new`, so `.env` files are supported during local SSR runs.

## Build Commands
- `cargo build --no-default-features --features=ssr` builds the SSR server binary.
- `cargo build --lib --no-default-features --features=hydrate --target wasm32-unknown-unknown` builds the client-side library without bundling.
- `cargo leptos build` is the best full build command; it successfully builds both the WASM frontend and SSR server in this repo.
- `cargo build` without explicit features is usually not the command you want; prefer explicit `ssr` or `hydrate` feature selection.
- `cargo leptos` reads `[package.metadata.leptos]` from `Cargo.toml`, including `style/main.css`, `public/`, and the target output directories.

## Run Commands
- `cargo leptos watch` is the likely best local dev loop for live-reloading full-stack Leptos work.
- `cargo leptos serve` is a reasonable option when you want the SSR app served with the Leptos pipeline.
- `cargo run --no-default-features --features=ssr` runs the Axum SSR binary directly.
- When running the SSR app directly, ensure the required env vars are present or startup will fail in `AppState::new`.
- The default listen address is `127.0.0.1:3000`, sourced from `SITE_ADDR` or Leptos config.

## Test Commands
- `cargo test` runs default tests; currently this repo has no unit or doc tests, but this remains the baseline command.
- `cargo test --no-default-features --features=ssr` runs SSR-targeted tests.
- `cargo test --lib --no-default-features --features=hydrate --target wasm32-unknown-unknown` is the direct hydrate-side test form.
- `cargo leptos test` is the best comprehensive test command here; it successfully runs both server and front test targets.
- `cargo test --doc` is valid if you add doctests later.

## Running A Single Test
- Run a single Rust test with `cargo test test_name -- --exact`.
- Run a single SSR test with `cargo test --no-default-features --features=ssr test_name -- --exact`.
- Run a single hydrate/lib test with `cargo test --lib --no-default-features --features=hydrate --target wasm32-unknown-unknown test_name -- --exact`.
- Use `cargo test test_name -- --nocapture` when you need printed output.
- `cargo test -- --list` lists discovered tests; it currently reports zero tests in this repo.
- If you add module-scoped tests, prefer exact names so agents do not accidentally run the full suite.

## Lint And Format Commands
- `cargo fmt` is the formatter of record.
- `cargo fmt -- --check` is the CI-friendly formatting check and currently reports formatting diffs in this repo.
- `cargo clippy --all-targets --all-features` is the broadest lint command and currently succeeds with at least one warning.
- `cargo clippy --no-default-features --features=ssr --all-targets` is useful when frontend-only targets are irrelevant.
- `cargo clippy --no-default-features --features=hydrate --lib --target wasm32-unknown-unknown` is useful for client-only review.
- Fix Clippy suggestions when they improve clarity, but follow existing project patterns first.

## Current Validation State
- `cargo leptos build` succeeds.
- `cargo leptos test` succeeds.
- `cargo clippy --all-targets --all-features` succeeds with a `clippy::single_match` warning in `src/api.rs`.
- `cargo fmt -- --check` fails because the repository is not fully rustfmt-clean right now.
- Agents should avoid large style-only rewrites unless the task explicitly includes formatting cleanup.

## Code Style: General Rust
- Follow rustfmt formatting, even if some current files still need cleanup.
- Prefer small, explicit functions over deeply clever abstractions.
- Keep feature-gated SSR code clearly isolated with `#[cfg(feature = "ssr")]` blocks or modules.
- Use `snake_case` for functions, variables, modules, and fields.
- Use `PascalCase` for structs, enums, and Leptos component functions.
- Keep constants `SCREAMING_SNAKE_CASE`, as seen with `SECONDARY_NAMESPACE`.
- Prefer owned `String` values in UI and shared payload types when data crosses async or serialization boundaries.

## Code Style: Imports
- Use grouped `use` statements rather than inline fully-qualified paths when it improves readability.
- Keep `crate::...` imports near the top and generally before third-party imports when touching existing files that do so.
- Preserve local file conventions: some files are not perfectly normalized yet, so avoid churn-only import reshuffles.
- Use nested imports for related items, for example `leptos_router::{ components::{...}, StaticSegment }`.
- Gate imports with `#[cfg(feature = "ssr")]` when they are SSR-only to keep hydrate builds clean.

## Code Style: Types And Data Modeling
- Shared request/response types live in `src/types.rs`; extend them there instead of redefining ad hoc payload structs.
- Derive only what is needed, but `Debug`, `Clone`, `Serialize`, and `Deserialize` are common defaults for shared data.
- Prefer enums for closed sets of states or kinds, such as `PayCodeStatus` and `PayCodeParamType`.
- Match the project’s existing naming when modifying enums, even if some variants are currently all-caps.
- Keep validation logic close to the type when practical; `CreatePayCodeRequest::validate()` is the existing pattern.
- Return concrete domain structs from server functions instead of loose JSON blobs when the frontend consumes typed data.

## Code Style: Leptos And Frontend
- Use `#[component]` functions returning `impl IntoView` for UI components and pages.
- Lean on `RwSignal`, `Signal::derive`, `Resource`, `Action`, and `ServerAction` for reactive state; follow the existing reactive style.
- Keep page composition in `src/pages/` and reusable primitives in `src/components/`.
- Prefer explicit event handlers with small closures over hidden magic.
- Preserve the existing hand-authored utility-class approach in `style/main.css`; do not introduce Tailwind casually.
- Reuse existing CSS tokens like `--bg-primary`, `--text-primary`, and `--border-color` instead of inventing one-off colors.
- Preserve the existing visual language: Urbanist typography, glassy cards, grid background, and light/dark token pairs.
- Check both desktop and mobile layouts when editing views; this app uses manual utility classes and limited responsive helpers.

## Code Style: SSR, Async, And State
- App-wide SSR state is centralized in `AppState`; prefer wiring new shared services through it rather than creating globals.
- Use `Arc` for shared server resources, matching the existing DB, Cloudflare client, and wallet setup.
- Keep environment parsing in `AppState::new` or closely related bootstrapping code.
- Prefer async functions returning `Result<_, _>` rather than panicking in request paths.
- For concurrent independent I/O, the codebase already uses `tokio::join!`; follow that pattern where it improves latency.
- Keep SSR-only API routes under `src/server/api.rs` and `src/server/nut24.rs` rather than mixing raw Axum handlers into frontend modules.

## Error Handling
- Use `Result`-based flows for fallible operations; do not hide errors silently.
- In Leptos server functions, convert failures to `ServerFnError` with user-meaningful messages.
- In Axum handlers, return explicit HTTP status codes plus JSON error bodies, as in `src/server/nut24.rs`.
- Log operational failures with `tracing::{error, warn, info}` when they matter for production diagnosis.
- Include context in logs, especially around DNS operations, payment redemption, and state initialization.
- Avoid `unwrap()` in request/response paths unless failure is truly impossible; there are a few existing uses, but new code should be more defensive.
- Use `expect(...)` only for startup invariants or context that truly must exist, and keep messages specific.

## Naming Conventions
- User-facing components use descriptive names like `NewPayCodePage`, `SuccessPage`, and `SearchPage`.
- Server function names are action-oriented, for example `get_app_config`, `lookup_bip353`, and `create_paycode_server`.
- Keep abbreviations aligned with the Bitcoin domain already used in the repo: `lno`, `sp`, `creq`, `bip21`, `bip353`.
- Prefer domain language over generic names: use `paycode`, `mint`, `domain`, `wallet`, and `resolution` when appropriate.
- For callback or signal names in UI, short names like `on_submit`, `error_msg`, and `is_busy` match existing style.

## Persistence And External Integrations
- Cloudflare DNS updates are handled via `CloudflareClient`; keep provider-specific logic there.
- Database persistence goes through `Db`; add read/write helpers there instead of embedding storage logic elsewhere.
- Wallet, mint, and payout logic belong in SSR modules, never hydrate-only code.
- When adding new external API calls, include timing/logging similar to the existing Cloudflare and DB helpers.
- Keep rollback behavior explicit when side effects happen in sequence; `src/server/nut24.rs` is the best existing example.

## Documentation And Rules Inventory
- There are no Cursor rules in `.cursor/rules/`.
- There is no repo-level `.cursorrules` file.
- There is no Copilot instruction file at `.github/copilot-instructions.md`.
- `README.md` documents the NixOS module, secrets, and runtime expectations.
- `SKILL.md` documents the public paycode API and should stay aligned with API behavior if endpoints change.

## Agent Guidance For Safe Changes
- Check feature flags before concluding code is unused; many modules are only compiled in SSR or hydrate builds.
- Prefer targeted edits over repo-wide formatting because the worktree is not currently rustfmt-clean.
- If you touch API payloads or validation, update both shared types and any UI/server call sites.
- If you add tests, document the exact single-test invocation pattern for the new test names.
- If you change environment requirements or HTTP routes, update `README.md`, `.env.example`, and `SKILL.md` when applicable.
- Validate with the narrowest relevant command first, then the broader build/test command if the change crosses layers.
