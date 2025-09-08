_: {
  projectRootFile = ".git/config";
  settings = {
    allow-missing-formatter = false;
  };
  programs = {
    alejandra.enable = true;
    rustfmt.enable = true;
    toml-sort.enable = true;
    shellcheck.enable = true;
    mdformat.enable = true;
    deadnix.enable = true;
    statix.enable = true;
  };
}
