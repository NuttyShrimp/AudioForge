{
  description = "Simple OAuth2 server for hackerspaces";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };
  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
  flake-utils.lib.eachDefaultSystem (system:
  let
    pkgs = import nixpkgs {
      inherit system;
      overlays = [ (import rust-overlay) ];
    };
  in
  with pkgs;
  {
    devShells.default = mkShell rec {
      name = "AudioForge";
      nativeBuildInputs = with pkgs; [
        pkg-config
        wrapGAppsHook

        libxkbcommon
        libGL
        fontconfig

        # wayland libraries
        wayland

        # x11 libraries
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libX11
      ];
      buildInputs = with pkgs; [
        (rust-bin.stable.latest.default.override { extensions = [ "rust-analyzer" "rust-src" ]; })
        openssl
        xorg.libxcb
        gtkd
        clang

        ffmpeg
        libclang
        stdenv.cc.cc
        glib
      ];
      LD_LIBRARY_PATH = "${with pkgs; lib.makeLibraryPath nativeBuildInputs}";
      LIBCLANG_PATH = pkgs.libclang.lib + "/lib";
    };
  });
}
