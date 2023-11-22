{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  packages = [
    pkgs.binaryen
  ];
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    # bulid advice
    mold
    udev alsa-lib vulkan-loader
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    libxkbcommon wayland # To use the wayland feature
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}