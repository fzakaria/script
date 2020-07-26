let
  moz_overlay = import (builtins.fetchTarball
    "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rust-stable = nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [ "rust-src" "rust-analysis" "rustfmt-preview" "rls-preview"];
  };
in with nixpkgs;
mkShell {
  name = "scriptr";
  buildInputs = [ rust-stable rustracer evcxr ];
  RUST_BACKTRACE = 0;
  RUST_SRC_PATH = "${rust-stable}/lib/rustlib/src/rust/src";

}
