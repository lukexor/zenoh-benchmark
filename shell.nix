let
  pkgs = (import <nixpkgs> {
    overlays = [
      (import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz"))
    ];
  });
  project_root = builtins.getEnv "PWD";
  inherit (pkgs) stdenv;
in stdenv.mkDerivation {
  name = "acme-bisf";
  buildInputs = with pkgs; [
    nats-server
    openssl
    pkg-config
    python312Packages.matplotlib
    czmq
    (rust-bin.stable.latest.default.override {
      extensions = [
        "rust-analyzer"
        "rust-src" # for rust-analyzer
      ];
    })
  ];
}
