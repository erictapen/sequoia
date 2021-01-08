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
      inherit (pkgs) buildRustCrate;
    }
  );
in
{
  sequoia-sq = crates.workspaceMembers.sequoia-sq.build;
  # sequoia-tool = crates.workspaceMembers.sequoia-tool.build;
}
