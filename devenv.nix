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
      cargoLock.lockFile = ./Cargo.nix.lock;
      postPatch = ''
        ln -s ${./Cargo.nix.lock} Cargo.lock
      '';
      buildPhase = ''
        runHook preBuild
        cargo build --release --examples
        runHook postBuild
      '';
      installPhase = ''
        runHook preInstall
        mkdir -p $out/bin
        ls -1 $src/examples | sed 's/\.rs$//' | \
          xargs -I{} sh -c 'cp target/release/examples/{} $out/bin/$(echo {} | sed 's/_/-/g')'
        runHook postInstall
      '';
      src = ./.;
      env = {
        GIT_REVISION = "devenv";
      };
    };
  };
}
