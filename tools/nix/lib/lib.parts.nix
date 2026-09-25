{
  self,
  inputs,
  lib,
  ...
}:
let
  fs = lib.fileset;
  repoRoot = ./../../..;
  repoRootFileset = lib.fileset.fromSource repoRoot;

  # Lib filesystem.
  libFS = {
    # The repository root directory (inside the Nix store).
    inherit repoRoot repoRootFileset;

    # The repopsitory root fileset to be used with the
    # `lib.fileset` library.

    /*
      Add the local files contained in the given filesets
      to the store by using the `rootPath` as `root` in `toSource`.
      The `rootPath` in `toSource` represents the toplevel directory in the generated
      `/nix/store/...` path.

      See [fileset.toSource]().

      # Inputs

      `filesets`

      : The fileset names in `components` or pure `Fileset`s to put into the store.

      # Type

      ```
      toSource:: { filesets = [String or Fileset...], ... } -> SourceLike
      ```
      :::
    */
    toSource =
      {
        filesets,
        root ? repoRoot,
      }:
      let
        # Unionify all filesets together.
        sum = fs.unions filesets;

        # Intersect with rootFileset to only include
        # what is in the root (no non-Git files).
        final = fs.intersection repoRootFileset sum;
      in
      fs.toSource {
        inherit root;
        fileset = final;
      };
  };

  # Lib import.
  libImport = import ./import.nix { inherit inputs self; };

  # Lib shell.
  libShell = import ./shell.nix { inherit inputs lib repoRoot; };

  # Lib toolchains.
  libToolchain = import ./toolchains.nix {
    inherit
      repoRoot
      inputs
      self
      lib
      ;
  };
in
{
  flake.lib = {
    import = libImport;
    shell = libShell;
    fs = libFS;
    toolchain = libToolchain;
  };
}
