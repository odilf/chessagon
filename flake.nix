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
        formatter = pkgs.nixfmt-rfc-style;

        devShells.default = pkgs.mkShell rec {
          buildInputs = [
            pkgs.rust-bin.beta.latest.default
            # .override
            # {
            #   extensions = [ "rust-src" ];
            #   targets = [ "wasm32-unknown-unknown" ];
            # }
            pkgs.rust-analyzer
            pkgs.cargo-nextest
            pkgs.trunk

            pkgs.jujutsu

            # # For eframe
            # ## misc. libraries
            # pkgs.openssl
            # pkgs.pkg-config

            # ## GUI libs
            # pkgs.libxkbcommon
            # pkgs.libGL
            # pkgs.fontconfig

            # ## wayland libraries
            # pkgs.wayland

            # ## x11 libraries
            # pkgs.xorg.libXcursor
            # pkgs.xorg.libXrandr
            # pkgs.xorg.libXi
            # pkgs.xorg.libX11

          ];

          # LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
        };
      }
    )
    // {
      nixosModules.default = import ./module.nix;
    };
}
