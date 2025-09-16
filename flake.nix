{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    SHID = {
      url = "github:Notarin/SHID";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    SHID,
    ...
  } @ inputs: let
    systems = ["x86_64-linux"];
    buildEachSystem = output: builtins.map output systems;
    mergeSystems = output: (
      builtins.foldl' (acc: elem: nixpkgs.lib.recursiveUpdate acc elem) {} (buildEachSystem output)
    );
  in
    mergeSystems (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        craneLib = crane.mkLib pkgs;
        utils = import ./nix/utils.nix (inputs // {inherit system pkgs lib craneLib SHID;});
      in {
        packages.${system} = rec {
          Snix-Bot = craneLib.buildPackage (
            {meta.mainProgram = "Snix-Bot";} // utils.cargo.cargoCommon
          );
          default = Snix-Bot;
        };

        devShells.${system}.default = craneLib.devShell {
          inherit (utils) shellHook;
          packages = utils.cargo.dependencies;
        };

        formatter.${system} = utils.treefmt-config.config.build.wrapper;

        checks.${system} = {
          formatting = utils.treefmt-config.config.build.check self;
          clippy = craneLib.cargoClippy ({
              cargoClippyExtraArgs = "-- --deny warnings";
            }
            // utils.cargo.cargoCommon);
        };
      }
    );
}
