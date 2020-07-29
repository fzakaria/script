{ stdenv, rustPlatform, lib, ... }:
rustPlatform.buildRustPackage rec {
  pname = "scriptr";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  buildInputs = [ ];

  cargoSha256 = "13gxfrc7vxhf32y0vcp8x6rcjxc1hsq81qj1l4p9qrj7899k617y";
  verifyCargoDeps = true;

  meta = with stdenv.lib; {
    description = "A reimplementation of script in Rust";
    homepage = "https://example.org/my-project";
    license = licenses.mit;
    platforms = platforms.linux;
    maintainers = [ "farid.m.zakaria@gmail.com" ];
  };
}
