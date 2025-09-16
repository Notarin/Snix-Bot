# Flake-wide utilities
{
  system,
  pkgs,
  lib,
  treefmt-nix,
  craneLib,
  SHID,
  ...
}: rec {
  # The script ran on devshell startup. Mostly just used for githooks.
  shellHook = lib.getExe SHID.packages.${system}.SHID;
  treefmt-config = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
  # Used to provide the root of the flake for expressions that may need it.
  # Intended to keep expressions cwd agnostic.
  flakePath = ../.;
  # Cargo build expression utilities. Dependencies, etc.
  cargo = import ./cargo.nix {inherit pkgs craneLib flakePath;};
}
