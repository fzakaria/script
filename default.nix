let
  moz_overlay = import (builtins.fetchTarball
    "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  nixpkgs =
    import <nixpkgs> { overlays = [ moz_overlay (import ./overlay.nix) ]; };
in nixpkgs.scriptr
