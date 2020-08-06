{ stdenv, rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "scriptr";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  buildInputs = [ ];

  cargoSha256 = "1d2rws00gbkgngvks4801j1lrsq4z8xdyz8j4z5xib8a3sg6lg3z";
  verifyCargoDeps = true;

  meta = with stdenv.lib; {
    description = "A reimplementation of script in Rust";
    homepage = "https://github.com/fzakaria/scriptr";
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = [ "farid.m.zakaria@gmail.com" ];
  };
}
