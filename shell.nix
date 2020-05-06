# let
#   moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
#   nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
# in
# with nixpkgs;
with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "moz_overlay_shell";
  buildInputs = with pkgs;[
    # nixpkgs.latest.rustChannels.nightly.rust
    rustc
    cargo
    pkgconfig
    gcc
    openssl
    capnproto
    nettle
    clang
    llvmPackages.libclang
    sqlite
    carnix
    nixUnstable
  ];
  LIBCLANG_PATH = "${llvmPackages.libclang}/lib";
}
