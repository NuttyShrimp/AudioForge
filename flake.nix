{
  description = "Simple OAuth2 server for hackerspaces";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = { nixpkgs, flake-utils, devshell, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ devshell.overlays.default ]; #(import rust-overlay)
    };
  in
  with pkgs;
  {
    devShells.default = pkgs.devshell.mkShell {
      name = "AudioForge";
      imports = [ "${devshell}/extra/language/rust.nix" ];
      packages = [
        openssl.dev
        pkg-config
        cargo-udeps
        cargo-watch
        cargo-limit
        gcc

        xorg.libxcb
      ];
      env = [
        {
          name = "LD_LIBRARY_PATH";
          value = with pkgs; lib.makeLibraryPath [
            libGL
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];
        }
      ];
    };
  });
}
