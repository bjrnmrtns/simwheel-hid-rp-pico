{
  description = "placepulse rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, rust-overlay, nixpkgs, utils, ... }:
  utils.lib.eachDefaultSystem (system: 
  let
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
                targets = ["thumbv6m-none-eabi"];
                extensions = ["rust-src"];
            })
            pkgs.rust-analyzer
            pkgs.flip-link
            pkgs.probe-rs
            pkgs.rustfmt
        ];

        shellHook = ''
          export PS1="(simwheel)$PS1";
          echo "Welcome to the Rust dev environment!";
        '';
      };

      packages.default = pkgs.stdenv.mkDerivation {
        pname = "simwheel";
        version = "0.1.0";
        src = ./.;
        buildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
                targets = ["thumbv6m-none-eabi"];
                extensions = ["rust-src"];
            })
            pkgs.rust-analyzer
            pkgs.flip-link
            pkgs.probe-rs
            pkgs.rustfmt
        ];
        buildPhase = ''
          cargo build --release
        '';
        installPhase = ''
          mkdir -p $out/bin
          cp target/release/placepulse $out/bin/
        '';
      };
    });
}

