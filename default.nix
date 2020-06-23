{ nixpkgs ? <nixpkgs> }:
let
  pkgs = import nixpkgs {
    config = {
      localSystem.system = "x86_64-linux";
    };
    system = "x86_64-linux";
  };
  crates = (
    import ./Cargo.nix {
      inherit pkgs;
      buildRustCrate = pkgs.buildRustCrate.override {
        defaultCrateOverrides = {
          capnp-rpc = attrs: { nativeBuildInputs = with pkgs; [ capnproto ]; };
          openssl-sys = attrs: {
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];
          };
          nettle-sys = attrs: {
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ nettle clang ];
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang}/lib";
          };
          sequoia-openpgp = attrs: { buildInputs = with pkgs; [ gmp ]; };
          sequoia-openpgp-ffi = attrs: { buildInputs = with pkgs; [ gmp ]; };
          sequoia-ipc = attrs: { buildInputs = with pkgs; [ gmp ]; };
          sequoia-guide = attrs: { buildInputs = with pkgs; [ gmp ]; };
          sequoia-store = attrs: {
            nativeBuildInputs = with pkgs; [ capnproto ];
            buildInputs = with pkgs; [ gmp sqlite ];
          };
          sequoia-ffi = attrs: {
            # sequoia-ffi includes a file from another crate...
            # I'm pretty sure what they are doing is wrong but I don't know how
            # to build (and upstream) a fix for that.
            src = pkgs.fetchFromGitLab {
              owner = "sequoia-pgp";
              repo = "sequoia";
              rev = "f10ceaa4ac04dcafe0bce42eb4fed3832225b594";
              sha256 = "sha256:19zzcz7v243l5k4mcfsyslxkglmii7agp611npz5b010m0r1x8sn";
            };
            sourceRoot = "source/ffi";
          };
          sequoia-tool = attrs: {
            # nativeBuildInputs = with pkgs; [ capnproto ];
            buildInputs = with pkgs; [ gmp sqlite ];
          };
        };
      };
    }
  );
in
{
  sequoia = crates.workspaceMembers.sequoia.build;
  sequoia-tool = crates.workspaceMembers.sequoia-tool.build;
}
