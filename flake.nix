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

          tabulator-rs =
            let cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            in pkgs.rustPlatform.buildRustPackage
              {
                pname = "tabulator-rs";
                version = cargo.package.version;

                src = ./.;

                cargoDeps = pkgs.rustPlatform.importCargoLock {
                  lockFile = ./Cargo.lock;
                };

                meta = with pkgs.lib; {
                  description = "Beancount frontend using Steel Scheme and Lima parser";
                  homepage = "https://github.com/tesujimath/beancount-lima";
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

          packages.default = tabulator-rs;

          apps = {
            tests = {
              type = "app";
              program = "${writeShellScript "tabulator-rs-tests" ''
                export PATH=${pkgs.lib.makeBinPath (ci-packages ++ [tabulator-rs])}
                just test
              ''}";
            };
          };
        }
      );
}
