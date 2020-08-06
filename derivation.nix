{ stdenv, rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "scriptr";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  buildInputs = [ ];

  cargoSha256 = "1vwir85qc0nla148x5xw8ind50fcr3gwfvaxdj24mihrl57a68gb";
  verifyCargoDeps = true;

  meta = with stdenv.lib; {
    description = "A reimplementation of script in Rust";
    homepage = "https://github.com/fzakaria/scriptr";
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = [ "farid.m.zakaria@gmail.com" ];
  };
}
