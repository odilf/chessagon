{
  description = "Hexagonal chess";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      rec {
        packages.default = pkgs.callPackage ./default.nix { };

        formatter = pkgs.nixfmt-rfc-style;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-bin.beta.latest.default
            pkgs.rust-analyzer
            pkgs.cargo-nextest

            pkgs.jujutsu
            packages.default.buildInputs
          ];
        };
      }
    )
    // {
      nixosModules.default = import ./module.nix;
    };
}
