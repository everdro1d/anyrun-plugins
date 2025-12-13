{ lib
, glib
, makeWrapper
, rustPlatform
, atk
, gtk3
, gtk-layer-shell
, pkg-config
, librsvg
, inputs
, name
, lockFile
, ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ./plugins/${name}/Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  version = cargoToml.package. version;

  src = "${inputs.self}";
  cargoLock = {
    inherit lockFile;
    outputHashes = {
      "anyrun-interface-25.12.0" = "09gi23v79xj61lndfwms9nd2knmprhxng7jx0bwy8c22yyv0j02i";
      "anyrun-macros-25.12.0" = "1i0vb1mw9mmq1agi1b1vdmycp3y2grls78k42iym71vdfif1vb4g";
    };
  };

  buildInputs = [
    glib
    atk
    gtk3
    librsvg
    gtk-layer-shell
  ];

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  doCheck = true;
  CARGO_BUILD_INCREMENTAL = "false";
  RUST_BACKTRACE = "full";
  copyLibs = true;
  cargoBuildFlags = [ "-p ${name}" ];
  buildAndTestSubdir = "plugins/${name}";

  meta = with lib; {
    description = "The ${name} plugin for Anyrun";
    homepage = "https://github.com/everdro1d/anyrun-plugins";
    license = with licenses; [ mit ];
  };
}
