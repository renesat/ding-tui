{
  pkgs,
  lib,
  ...
}: {
  autoWire = ["crate" "clippy" "doc"];
  crane.args = {
    buildInputs =
      [pkgs.openssl]
      ++ lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks; [
          IOKit
        ]
      );
  };
}
