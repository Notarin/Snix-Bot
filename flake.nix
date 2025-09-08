{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    treefmt-nix,
    ...
  }: (
    builtins.foldl' (acc: elem: nixpkgs.lib.recursiveUpdate acc elem) {} (
      builtins.map (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
          craneLib = crane.mkLib pkgs;

          # Flake-wide utilities
          utils = let
            utilsDir = ./nix;
          in {
            shellHook = builtins.readFile "${utilsDir}/shellHook.sh";
            treefmt-config = treefmt-nix.lib.evalModule pkgs "${utilsDir}/treefmt.nix";
            flakePath = ./.;
          };

          src = ./.;
          cargoArtifacts = craneLib.buildDepsOnly {
            inherit src;
          };
        in {
          packages.${system} = rec {
            Nix = craneLib.buildPackage {
              inherit src;
              nativeBuildInputs = with pkgs; [
                pkg-config
              ];
              meta = {
                mainProgram = "Snix-Bot";
              };
            };
            default = Nix;
          };
          devShells.${system}.default = craneLib.devShell {
            inherit (utils) shellHook;
            packages = with pkgs; [
              pkg-config
            ];
          };
          formatter.${system} = utils.treefmt-config.config.build.wrapper;
          checks.${system} = {
            formatting = utils.treefmt-config.config.build.check self;
            clippy = craneLib.cargoClippy {
              inherit cargoArtifacts src;
              cargoClippyExtraArgs = "-- --deny warnings";
            };
          };
        }
      )
      [
        "x86_64-linux"
      ]
    )
  );
}
