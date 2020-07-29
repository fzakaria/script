let
  pkgs =
    import <nixpkgs> {};
in with pkgs;
mkShell {
  name = "scriptr";
  buildInputs = [rustc cargo evcxr rustracer];
  RUST_BACKTRACE = 1;
}