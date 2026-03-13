{ config, lib, pkgs, ... }:

let
  cfg = config.services.nutty;

  domainsJson = builtins.toJSON cfg.domains;
  acceptedMints = lib.concatStringsSep "," cfg.acceptedMints;
  siteAddr = "${cfg.host}:${toString cfg.port}";
in
{
  options.services.nutty = {
    enable = lib.mkEnableOption "the Nutty SSR application";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.nutty;
      defaultText = lib.literalExpression "pkgs.nutty";
      description = "Package providing the Nutty server binary and site assets.";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      example = "0.0.0.0";
      description = "Host address the Nutty server listens on.";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 3000;
      description = "Port the Nutty server listens on.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      example = "/run/secrets/nutty.env";
      description = ''
        Path to an environment file containing sensitive values such as
        `CF_TOKEN` and `CDK_MNEMONIC`.
      '';
    };

    cfToken = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "your_cloudflare_token";
      description = ''
        Cloudflare API token passed directly to the service environment.
        This is less secure than `environmentFile` because the value will end
        up in the Nix store.
      '';
    };

    cdkMnemonic = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12";
      description = ''
        Wallet mnemonic passed directly to the service environment.
        This is less secure than `environmentFile` because the value will end
        up in the Nix store.
      '';
    };

    appName = lib.mkOption {
      type = lib.types.str;
      default = "Nutty";
      description = "Application name shown in the UI and used for the state directory prefix.";
    };

    defaultDomain = lib.mkOption {
      type = lib.types.str;
      default = "nutty.cash";
      description = "Default domain shown in the UI.";
    };

    appUrl = lib.mkOption {
      type = lib.types.str;
      default = "https://nutty.cash";
      example = "https://pay.example.com";
      description = "Public base URL of the Nutty instance.";
    };

    network = lib.mkOption {
      type = lib.types.str;
      default = "bitcoin";
      example = "signet";
      description = "Bitcoin network name passed to the application.";
    };

    rustLog = lib.mkOption {
      type = lib.types.str;
      default = "info";
      example = "info,nutty=debug,cdk=debug";
      description = ''
        Value for the `RUST_LOG` environment variable used by the tracing
        subscriber.
      '';
    };

    acceptedMints = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      example = [ "https://mint.example.com" ];
      description = "Accepted Cashu mint base URLs.";
    };

    payoutCreq = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "creqA...";
      description = ''
        Optional amountless Cashu payment request used for best-effort payout of
        newly received ecash. The application sends the wallet's total balance and
        keeps the ecash in the wallet if the payout attempt fails.
      '';
    };

    domains = lib.mkOption {
      type = lib.types.attrsOf lib.types.str;
      default = { };
      example = {
        "example.com" = "cloudflare-zone-id";
      };
      description = "Mapping from domain names to Cloudflare zone IDs.";
    };

    customPriceSats = lib.mkOption {
      type = lib.types.ints.unsigned;
      default = 5000;
      description = "Price in sats for a custom paycode.";
    };

    randomPriceSats = lib.mkOption {
      type = lib.types.ints.unsigned;
      default = 0;
      description = "Price in sats for a randomly generated paycode.";
    };
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.acceptedMints != [ ];
        message = "services.nutty.acceptedMints must contain at least one mint URL.";
      }
      {
        assertion = cfg.environmentFile != null || (cfg.cfToken != null && cfg.cdkMnemonic != null);
        message = "Set services.nutty.environmentFile, or set both services.nutty.cfToken and services.nutty.cdkMnemonic.";
      }
    ];

    users.users.nutty = {
      isSystemUser = true;
      group = "nutty";
      home = "/var/lib/nutty";
      createHome = true;
    };

    users.groups.nutty = { };

    systemd.services.nutty = {
      description = "Nutty SSR application";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];

      environment = {
        HOME = "/var/lib/nutty";
        APP_NAME = cfg.appName;
        DEFAULT_DOMAIN = cfg.defaultDomain;
        APP_URL = cfg.appUrl;
        NETWORK = cfg.network;
        RUST_LOG = cfg.rustLog;
        ACCEPTED_MINTS = acceptedMints;
        DOMAINS = domainsJson;
        CUSTOM_PRICE_SATS = toString cfg.customPriceSats;
        RANDOM_PRICE_SATS = toString cfg.randomPriceSats;
        SITE_ADDR = siteAddr;
      } // lib.optionalAttrs (cfg.payoutCreq != null) {
        PAYOUT_CREQ = cfg.payoutCreq;
      } // lib.optionalAttrs (cfg.cfToken != null) {
        CF_TOKEN = cfg.cfToken;
      } // lib.optionalAttrs (cfg.cdkMnemonic != null) {
        CDK_MNEMONIC = cfg.cdkMnemonic;
      };

      serviceConfig = {
        Type = "simple";
        User = "nutty";
        Group = "nutty";
        EnvironmentFile = cfg.environmentFile;
        StateDirectory = "nutty";
        WorkingDirectory = cfg.package;
        ExecStart = "${cfg.package}/bin/nutty";
        Restart = "on-failure";
        RestartSec = 5;
        AmbientCapabilities = [ ];
        CapabilityBoundingSet = [ ];
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "strict";
        ReadWritePaths = [ "/var/lib/nutty" ];
        RemoveIPC = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        UMask = "0077";
      };
    };
  };
}
