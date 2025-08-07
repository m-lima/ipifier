{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    helper.url = "github:m-lima/nix-template";
  };

  outputs =
    {
      self,
      flake-utils,
      helper,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      (helper.lib.rust.helper inputs system ./. {
        buildInputs = pkgs: [ pkgs.openssl ];
        nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
      }).outputs
    )
    // {
      nixosModules =
        let
          module =
            {
              lib,
              config,
              pkgs,
              ...
            }:
            let
              cfg = config.ipifier;
            in
            {
              options = {
                ipfier = {
                  enable = lib.mkEnableOption "dynamic cloudflare updater";
                  configuration = lib.mkOption {
                    type = lib.types.path;
                    description = "Path to the configuration file";
                    example = "./options.json";
                  };

                  period = lib.mkOption {
                    type = lib.types.singleLineStr;
                    description = "Period to run the update";
                    default = "*:0/10";
                  };
                };
              };

              config = lib.mkIf cfg.enable {
                systemd = {
                  services.ipifier = {
                    description = "Checks and updates the system's public IP against CloudFlare";
                    serviceConfig = {
                      Type = "oneshot";
                    };

                    wantedBy = [ "multi-user.target" ];
                    script = "${self.packages.${pkgs.system}.default}/bin/ipifier ${cfg.configuration}";
                  };
                  timers.ipifier = {
                    wantedBy = [ "timers.target" ];
                    timerConfig = {
                      OnBootSec = cfg.period;
                      OnUnitActiveSec = cfg.period;
                      Persistent = true;
                    };
                  };
                };
              };
            };
        in
        {
          default = module;
          ipifier = module;
        };
    };
}
