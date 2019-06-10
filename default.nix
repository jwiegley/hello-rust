{ mozrev  ? "50bae918794d3c283aeb335b209efd71e75e3954"
, mozsha  ? "07b7hgq5awhddcii88y43d38lncqq9c8b2px4p93r5l7z0phv89d"
, mozilla ? import (builtins.fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/${mozrev}.tar.gz";
    sha256 = mozsha;
  })

, rev    ? "61f0936d1cd73760312712615233cd80195a9b47"
, sha256 ? "1fkmp99lxd827km8mk3cqqsfmgzpj0rvaz5hgdmgzzyji70fa2f8"
, pkgs   ?
  import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = true;
    config.packageOverrides = pkgs: rec {
      rls = pkgs.rls.overrideDerivation (attrs: {
        buildInputs = attrs.buildInputs ++
          (pkgs.stdenv.lib.optional pkgs.stdenv.isDarwin
             pkgs.darwin.apple_sdk.frameworks.Security);
      });
    };
    overlays = [ mozilla ];
  }

, mkDerivation ? null
}:

with pkgs;

let
  nightly = rustChannelOf {
    date = "2019-05-07";
    channel = "nightly";
  };
  stable = rustChannelOf {
    channel = "stable";
  };
  rustPlatform = makeRustPlatform {
    rustc = stable.rust;
    cargo = stable.rust;
  };
in

rustPlatform.buildRustPackage rec {
  pname = "hello";
  version = "1.0.0";

  src = ./.;

  cargoSha256 = "13wjwicd1xrhzhrdx89r5ghz74la4m1jx5s74zwifmc57lq5y57k";
  cargoSha256Version = 2;
  cargoBuildFlags = [];

  nativeBuildInputs = [ asciidoc asciidoctor plantuml docbook_xsl libxslt ];
  buildInputs = [ cargo rustfmt carnix ]
    ++ (stdenv.lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  preFixup = ''
  '';

  meta = with stdenv.lib; {
    description = "Hello, world!";
    homepage = https://github.com/jwiegley/hello-rust;
    license = with licenses; [ mit ];
    maintainers = [ maintainers.jwiegley ];
    platforms = platforms.all;
  };
}
