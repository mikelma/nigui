# { pkgs ? import <nixpkgs> {},
#   lib ? import <nixpkgs/lib> {} }:
# { lib, nixpkgs, ... }:

let
  pkgs = import <nixpkgs> {};
  lib = import <nixpkgs/lib>;
in
  pkgs.mkShell rec {
    buildInputs = with pkgs; [
      # rust related
      rustc
      cargo
      rustfmt
      rust-analyzer
      clippy

      # other dependencies
      dbus
      pkgconfig
      xorg.libxcb
      libxkbcommon
      xorg.libX11
      libGL
      wayland
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
    ];

    LD_LIBRARY_PATH = "${lib.strings.makeLibraryPath buildInputs}";
    RUST_BACKTRACE = 1;
  }
