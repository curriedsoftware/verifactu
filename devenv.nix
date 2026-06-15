{
  pkgs,
  lib,
  config,
  inputs,
  ...
}: {
  languages = {
    c.enable = true;
    rust.enable = true;
  };

  packages = with pkgs; [alejandra bat cargo-audit cargo-deny libxml2 just jq];

  enterTest = ''
    cargo test
    cargo clippy
    cargo deny check
    cargo audit -f ${./Cargo.nix.lock} --json | ${lib.getExe pkgs.jq} -e '. as $expression | $expression, ($expression | .vulnerabilities.found | not)'
  '';

  git-hooks.hooks = {
    rustfmt.enable = true;
    clippy.enable = true;
    trufflehog.enable = true;
  };

  outputs = {
    verifactu = pkgs.rustPlatform.buildRustPackage {
      name = "verifactu";
      src = ./.;
      # Build from the committed ./vendor sources (offline). Cargo.lock is
      # gitignored; symlink the tracked Cargo.nix.lock into place for cargo.
      cargoVendorDir = "vendor";
      cargoLock = {
        lockFile = ./Cargo.nix.lock;
        allowBuiltinFetchGit = true;
      };
      postPatch = ''
        ln -sf ${./Cargo.nix.lock} Cargo.lock
      '';
      cargoBuildFlags = ["--bin" "verifactu"];
      # The test-suite talks to the AEAT web service / replays recorded
      # responses, so it can't run inside the sandboxed build.
      doCheck = false;
      meta.mainProgram = "verifactu";
    };
  };
}
