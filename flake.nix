{
  description = "Rust tabulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs {
            inherit system overlays;
          };
          # cargo-nightly based on https://github.com/oxalica/rust-overlay/issues/82
          nightly = pkgs.rust-bin.selectLatestNightlyWith (t: t.default);
          cargo-nightly = pkgs.writeShellScriptBin "cargo-nightly" ''
            export RUSTC="${nightly}/bin/rustc";
            exec "${nightly}/bin/cargo" "$@"
          '';


          ci-packages = with pkgs; [
            bashInteractive
            coreutils
            diffutils
            elvish
            just
            rust-bin.stable.latest.default
            gcc
          ];

          tabulator =
            let cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            in pkgs.rustPlatform.buildRustPackage
              {
                pname = "tabulator";
                version = cargo.package.version;

                src = ./.;

                cargoDeps = pkgs.rustPlatform.importCargoLock {
                  lockFile = ./Cargo.lock;
                };

                cargoBuildFlags = [ "--features=bin" ];

                meta = with pkgs.lib; {
                  description = "Grid-oriented tabulation with justification and anchors";
                  homepage = "https://github.com/tesujimath/tabulator";
                  license = with licenses; [ asl20 mit ];
                  # maintainers = [ maintainers.tesujimath ];
                };
              };
        in
        with pkgs;
        {
          devShells.default = mkShell {
            nativeBuildInputs = [
              cargo-modules
              cargo-nightly
              cargo-udeps
              cargo-outdated
              cargo-edit
              gdb
            ] ++ ci-packages;

            shellHook = ''
              PATH=$PATH:$(pwd)/target/debug
            '';
          };

          packages.default = tabulator;

          apps = {
            tests = {
              type = "app";
              program = "${writeShellScript "tabulator-tests" ''
                export PATH=${pkgs.lib.makeBinPath (ci-packages ++ [tabulator])}
                just test
              ''}";
            };
          };
        }
      );
}
