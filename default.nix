let
  nixpkgs =
    import <nixpkgs> { overlays = [ (import ./overlay.nix) ]; };
in nixpkgs.scriptr
