{
    inputs = {
        nixpkgs.url = "nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };
    outputs = { self, nixpkgs, flake-utils, rust-overlay }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs {
                    inherit system overlays;
                };
                rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
                rustPlatform = pkgs.makeRustPlatform {
                    cargo = rustToolchain;
                    rustc = rustToolchain;
                };
                meta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
            in
            with pkgs; {
                devShells.default = mkShell {
                    buildInputs = [rustToolchain rust-analyzer];
                };
                packages.default = rustPlatform.buildRustPackage {
                    pname = meta.name;
                    version = meta.version;
                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;
                };
            }
        );
}
