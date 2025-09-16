{
  pkgs,
  craneLib,
  flakePath,
  ...
}: rec {
  src = flakePath;
  # Dependencies needed to build the project.
  # Will be provided to packages, devshell(s), checks, plus more if needed.
  dependencies = with pkgs; [
    pkg-config
    openssl
  ];
  nativeBuildInputs = dependencies; # Alias
  # Dependency crates derivation. Makes the same deps build reusable.
  cargoArtifacts = craneLib.buildDepsOnly {
    inherit src nativeBuildInputs;
  };
  # An attrSet which is added to any cargo build drv args.
  cargoCommon = {
    inherit src cargoArtifacts nativeBuildInputs;
  };
}
