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
```

The module creates a `nutty` service account, stores runtime state in `/var/lib/nutty`,
and starts the SSR app on the configured `host` and `port`.

## Credits

Inspired by [twelvecash](https://github.com/ATLBitLab/twelvecash) by ATL BitLab.
