{
  ...
}:
{
  perSystem =
    {
      self',
      config,
      pkgs,
      ...
    }:
    let
      cfg = config;
    in
    {
      repo.toolchains.rust = [
        (
          { config, lib, ... }:
          let
            toolchain = cfg.repo.pinned.rust.shell.toolchain;
          in
          {
            packages = [
              toolchain
              pkgs.cargo-watch

              # Debugging
              pkgs.lldb_18
            ];

            languages.rust = {
              enable = true;

              # https://github.com/cachix/devenv/issues/3182
              toolchainPackage = toolchain;
              lsp.package = toolchain;
            };

            env = {
              CARGO_TARGET_DIR = "${config.devenv.root}/.output/build";
              RUST_SRC_PATH = lib.mkForce "${toolchain}/lib/rustlib/src/rust/library";
            };
          }
        )
      ];
    };
}
