{
  ...
}:
{
  perSystem =
    { pkgs, ... }:
    {
      # The shell modules (devenv) for a generic shell.
      repo.toolchains.generic = [
        {
          packages = [ pkgs.cowsay ];
        }
      ];
    };
}
