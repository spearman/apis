with import <nixpkgs> {};
mkShell {
  buildInputs = [
    cargo-udeps
    gdb # required for rust-gdb
    gh
    rustup
    rust-analyzer
    yamllint
  ];
  # required for opengl interactive example
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    libglvnd
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
  ];
}
