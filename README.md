# Nutty

Bitcoin Payments, Human-Friendly. Powered by Lightning and Cashu.

## NixOS Module

This repository now exports an app-only NixOS module via `nixosModules.default`.

Example flake input:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nutty.url = "git+ssh://git@github.com/thesimplekid/nutty.cash.git";
  };

  outputs = { self, nixpkgs, nutty, ... }: {
    nixosConfigurations.host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        nutty.nixosModules.default
        ({ ... }: {
          services.nutty = {
            enable = true;
            host = "127.0.0.1";
            port = 3000;
            appName = "Nutty";
            defaultDomain = "nutty.cash";
            appUrl = "https://nutty.cash";
            network = "bitcoin";
            acceptedMints = [ "https://mint.example.com" ];
            payoutCreq = "creqA...";
            domains = {
              "example.com" = "cloudflare-zone-id";
            };
            environmentFile = "/run/secrets/nutty.env";
          };
        })
      ];
    };
  };
}
```

The environment file should contain secrets such as:

```env
CF_TOKEN=your_cloudflare_token
CDK_MNEMONIC=your seed words here
PAYOUT_CREQ=creqA... # optional, must be amountless
```

If `PAYOUT_CREQ` is set, Nutty will attempt a best-effort payout after receiving ecash by
sending the wallet's total current balance to that Cashu payment request. The request must be
amountless so the runtime can provide the balance dynamically. If the payout fails, the paycode
creation still succeeds and the ecash remains in the wallet so it can be retried later.

For quick local setups, you can also define the values directly in Nix instead
of using an environment file:

```nix
services.nutty = {
  enable = true;
  cfToken = "your_cloudflare_token";
  cdkMnemonic = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12";
};
```

This is convenient for testing, but it is not secure because the secrets will
be stored in the Nix store.

The module creates a `nutty` service account, stores runtime state in `/var/lib/nutty`,
and starts the SSR app on the configured `host` and `port`.

## Credits

Inspired by [twelvecash](https://github.com/ATLBitLab/twelvecash) by ATL BitLab.
