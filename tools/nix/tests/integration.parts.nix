{ ... }: {
  perSystem =
    { pkgs, ... }:
    let
      config = import ./configurations.nix { };
    in
    {
      packages.integration-test = pkgs.testers.runNixOSTest {
        name = "integration-test";

        inherit (config) nodes;

        testScript =
          # Python
          ''
            from dataclasses import dataclass
            @dataclass
            class Machines:
                side_a: Any
                side_b: Any

            log.info("Starting tests.")
            start_all()

            log.info("Create test file.")
            side_a.succeed("touch /tmp/shared/test")

            log.info("Read test file.")
            side_b.succeed("ls -al /tmp/shared/test")
          '';
      };
    };
}
