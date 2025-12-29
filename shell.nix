{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.gnuplot
    pkgs.python3
  ];
  shellHook = ''
    export PATH="$PWD/tools:$PATH"
    export NIX_ENFORCE_PURITY=0
    bench_plot.sh
  '';
}
