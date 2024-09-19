{
  pkgs,
  lib,
  ...
}: {
  autoWire = ["crate" "clippy" "doc"];
  crane.args = {
    buildInputs = lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.apple_sdk_frameworks; [
        IOKit
      ]
    );
  };
}
