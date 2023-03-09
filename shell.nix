{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.cargo
    pkgs.cargo-flamegraph
    pkgs.clippy
    pkgs.gcc
    pkgs.gdb
    pkgs.rust-analyzer
    pkgs.rustc
    pkgs.rustfmt
  ];
}
