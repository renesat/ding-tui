{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";
    rust-flake.url = "github:juspay/rust-flake";
    rust-flake.inputs.nixpkgs.follows = "nixpkgs";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    cargo-doc-live.url = "github:srid/cargo-doc-live";

    # Dev tools
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import inputs.systems;
      imports = [
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
        inputs.process-compose-flake.flakeModule
        inputs.cargo-doc-live.flakeModule
        inputs.pre-commit-hooks.flakeModule
      ];
      perSystem = {
        config,
        self',
        pkgs,
        ...
      }: {
        cargo-doc-live.crateName = "ding-rs";

        rust-project = {
          crateNixFile = "crate.nix";
        };

        pre-commit.settings.hooks = {
          # FIXME: Not working with `nix flake check`. Wait until merge https://github.com/cachix/git-hooks.nix/pull/396
          clippy = {
            enable = true;
            packageOverrides.cargo = (config.rust-project.crane-lib.packages {}).cargo;
            packageOverrides.clippy = (config.rust-project.crane-lib.packages {}).clippy;
          };
          rustfmt.enable = true;
          alejandra.enable = true;
          deadnix = {
            enable = true;
            args = ["--edit"];
          };
          statix = {
            enable = true;
            settings = {
              format = "stderr";
            };
          };
          ripsecrets.enable = true;
          nil.enable = true;
          flake-checker.enable = true;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [
            self'.devShells.rust
          ];
          packages = [
            pkgs.just
            pkgs.cargo-watch
            config.process-compose.cargo-doc-live.outputs.package
            config.rust-project.crates.ding-rs.crane.args.buildInputs
          ];
          shellHook = config.pre-commit.installationScript;
        };
        packages.default = self'.packages.ding-rs;
      };
    };
}
