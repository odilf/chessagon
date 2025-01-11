{
  lib,
  rustPlatform,
  stdenv,
  darwin,
  libiconv,
  openssl,
  pkg-config,
}:
let
  config-toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in

rustPlatform.buildRustPackage {
  pname = config-toml.package.name;
  version = config-toml.package.version;

  src = lib.sources.cleanSource ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkg-config ];

  buildInputs =
    [ openssl ]
    ++ lib.optionals stdenv.hostPlatform.isDarwin [
      darwin.apple_sdk.frameworks.SystemConfiguration
      libiconv
    ];
}
