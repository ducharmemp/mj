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
      in
      with pkgs;
      {
        formatter = pkgs.nixpkgs-fmt;
        devShells.default = mkShell {
          buildInputs = with toolchain; [ (withComponents [ "cargo" "clippy" "rust-src" "rustc" "rustfmt" ]) rust-analyzer-nightly ];
          RUST_BACKTRACE = 1;
        };
      });
}
