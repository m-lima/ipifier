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
      let
        outputs =
          (helper.lib.rust.helper inputs system ./. {
            buildInputs = pkgs: [ pkgs.openssl ];
            nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
          }).outputs;
      in
      outputs
      // {
        packages = outputs.packages // {
          noTimestamp =
            (helper.lib.rust.helper inputs system ./. {
              noDefaultFeatures = true;
              buildInputs = pkgs: [ pkgs.openssl ];
              nativeBuildInputs = pkgs: [ pkgs.pkg-config ];
            }).outputs.packages.default;
        };
      }
    );
}
