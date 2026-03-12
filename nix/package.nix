{ lib
, rustPlatform
, makeBinaryWrapper
, cargo-leptos
, wasm-bindgen-cli
, dart-sass
, binaryen
, llvmPackages
, pkg-config
, openssl
, stdenv
}:

rustPlatform.buildRustPackage rec {
  pname = "nutty";
  version = "0.1.0";

  src = lib.cleanSource ../.;
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = {
      "cashu-0.15.1" = "sha256-rwR+IS72cJTj4x5Mps9OhqJk+Wek2SyZvZHIU3Ounh0=";
    };
  };

  nativeBuildInputs = [
    cargo-leptos
    wasm-bindgen-cli
    dart-sass
    binaryen
    llvmPackages.lld
    pkg-config
    makeBinaryWrapper
  ];

  buildInputs = [ openssl ];

  cargoBuildFlags = [ "--bin" "nutty" "--no-default-features" "--features" "ssr" ];

  buildPhase = ''
    runHook preBuild
    export HOME="$TMPDIR/home"
    mkdir -p "$HOME"
    cargo leptos build --release --bin-features ssr --lib-features hydrate
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin $out/share/nutty

    install -m755 target/release/nutty $out/bin/nutty-unwrapped
    cp -r target/site/. $out/share/nutty/

    makeBinaryWrapper $out/bin/nutty-unwrapped $out/bin/nutty \
      --set LEPTOS_SITE_ROOT $out/share/nutty

    runHook postInstall
  '';

  meta = with lib; {
    description = "Bitcoin payments app powered by Lightning and Cashu";
    homepage = "https://github.com/thesimplekid/nutty.cash";
    license = licenses.mit;
    platforms = platforms.linux ++ platforms.darwin;
    mainProgram = "nutty";
  };
}
