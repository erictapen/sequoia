{
  description = "Sequoia";

  inputs.nixpkgs.url = "github:erictapen/Nixpkgs/sequoia-crate-overrides";

  outputs =
    { self
    , nixpkgs
    }: rec {

      packages.x86_64-linux.sequoia-sq = (
        import ./default.nix {
          inherit nixpkgs;
        }
      ).sequoia-sq;

      # packages.x86_64-linux.sequoia-tool = (
      #   import ./default.nix {
      #     inherit nixpkgs;
      #   }
      # ).sequoia-tool;

      defaultPackage.x86_64-linux = packages.x86_64-linux.sequoia-sq;

    };

}
