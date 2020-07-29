{ stdenv, rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "scriptr";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  buildInputs = [ ];

  cargoSha256 = "1aa3yz8jl244c1ja5sf8bhlngi8zpc46czl88553p9hnivbrpkw1";
  verifyCargoDeps = true;

  meta = with stdenv.lib; {
    description = "A reimplementation of script in Rust";
    homepage = "https://example.org/my-project";
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = [ "farid.m.zakaria@gmail.com" ];
  };
}
