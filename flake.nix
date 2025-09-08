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
        in {
          packages.${system} = rec {
            Nix = craneLib.buildPackage {
              src = ./.;
              nativeBuildInputs = with pkgs; [
                pkg-config
              ];
              meta = {
                mainProgram = "Nix";
              };
            };
            default = Nix;
          };
          devShells.${system}.default = craneLib.devShell {
            shellHook = ''
              oldHookDir=$(git config --local core.hooksPath)

              if [ "$oldHookDir" != "$PWD/.githooks" ]; then
                read -rp "Set git hooks to $PWD/.githooks? (y/n) " answer
                if [ "$answer" = "y" ]; then
                  git config core.hooksPath "$PWD"/.githooks
                  echo "Set git hooks to $PWD/.githooks"
                else
                  echo "Skipping git hooks setup"
                fi
              fi
            '';
            packages = with pkgs; [
              pkg-config
            ];
          };
          formatter.${system} = utils.treefmt-config.config.build.wrapper;
          checks.${system}.formatting = utils.treefmt-config.config.build.check self;
        }
      )
      [
        "x86_64-linux"
      ]
    )
  );
}
