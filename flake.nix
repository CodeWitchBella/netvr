{
  description = "Gemma";

  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.systems.follows = "systems";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-utils, devshell, nixpkgs, rust-overlay, ... }:
  flake-utils.lib.eachDefaultSystem
    (system:
      let
        overlays = [
          (import rust-overlay)
          devshell.overlays.default
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs.stdenv) isLinux;

        rust-toolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./netvr-rust/rust-toolchain.toml;
        
        packages = with pkgs; [
          rust-toolchain
          wasm-pack
        ];
      in {
        devShells.default = pkgs.devshell.mkShell {
          imports = [ (pkgs.devshell.importTOML ./devshell.toml) ];
          devshell.packages = packages;
        };
      }
    );
}