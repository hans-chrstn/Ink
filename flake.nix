{
  description = "Ink";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux"];

    forAllSystems = function:
      nixpkgs.lib.genAttrs supportedSystems (
        system:
          function (import nixpkgs {
            inherit system;
          })
      );
  in {
    packages = forAllSystems (
      pkgs: let
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            wrapGAppsHook4
          ];

          buildInputs = with pkgs; [
            gtk4
            gtk4-layer-shell
            glib
            pango
            gdk-pixbuf
            cairo
            graphene
            openssl
            libpulseaudio
            mako
          ];
        };
      }
    );

    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
        inputsFrom = [self.packages.${pkgs.stdenv.hostPlatform.system}.default];
        packages = with pkgs; [
          cargo
          rustc
          rust-analyzer
          clippy
          rustfmt
        ];
      };
    });

    nixosModules.default = {
      config,
      lib,
      pkgs,
      ...
    }: let
      cfg = config.programs.ink;
    in {
      options.programs.ink = {
        enable = lib.mkEnableOption "Ink";
        package = lib.mkOption {
          type = lib.types.package;
          default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
          description = "The Ink package to install.";
        };
      };

      config = lib.mkIf cfg.enable {
        environment.systemPackages = [cfg.package];
      };
    };

    homeModules.default = {
      config,
      lib,
      pkgs,
      ...
    }: let
      cfg = config.programs.ink;
    in {
      options.programs.ink = {
        enable = lib.mkEnableOption "Enable Ink - Widget Layer Shell Framework";

        package = lib.mkOption {
          type = lib.types.package;
          default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        };

        config = lib.mkOption {
          type = lib.types.nullOr lib.types.lines;
          default = null;
          description = "Content of main.lua";
        };

        modules = lib.mkOption {
          type = lib.types.attrsOf lib.types.lines;
          default = {};
          description = "Extra Lua modules to put in ~/.config/ink/modules/";
        };
      };

      config = lib.mkIf cfg.enable {
        home.packages = [cfg.package];

        xdg.configFile = let
          mainConfig =
            if cfg.config != null
            then {
              "ink/main.lua".text = cfg.config;
            }
            else {};

          moduleConfigs =
            lib.mapAttrs' (
              name: value:
                lib.nameValuePair "ink/modules/${name}.lua" {text = value;}
            )
            cfg.modules;
        in
          mainConfig // moduleConfigs;
      };
    };
  };
}
