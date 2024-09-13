{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
  };
  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        toolchain = fenix.packages."${system}".complete;
        buildPackages = with pkgs; [
          pkg-config
          openssl
        ];
      in
      with pkgs;
      {
        formatter = pkgs.nixpkgs-fmt;
        devShells.default = mkShell {
          buildInputs = with toolchain; [ (withComponents [ "cargo" "clippy" "rust-src" "rustc" "rustfmt" ]) statix rust-analyzer-nightly ] ++ buildPackages;
          LD_LIBRARY_PATH = lib.makeLibraryPath buildPackages;
          RUST_BACKTRACE = 1;
        };
      });
}
