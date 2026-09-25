{ ... }: {
  perSystem = { self, pkgs, ... }: {
    packages.service = pkgs.callPackage ./service.nix {
      inherit self;
      rustPlatform = self.repo.pinned.rust;
    };
  };
}
