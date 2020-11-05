with import <nixpkgs> {};
mkShell {
  buildInputs = [
    gdb # required for rust-gdb
    rustup
    rust-analyzer
  ];
  # required for opengl interactive example
  LD_LIBRARY_PATH = stdenv.lib.makeLibraryPath [
    libglvnd
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
}
