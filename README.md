# Anyrun plugins

This repo is structured based on [wuliuqii's anyrun-plugins](https://github.com/wuliuqii/anyrun-plugins/).

## Installation

Add the flake:
```nix
# flake.nix
{
  inputs = {
    ...

    anyrun-plugins = {
      url = "github:everdro1d/anyrun-plugins";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };

    ...
  };
}
```

The flake provides packages for each plugin.

Add to anyrun's home-manager module:
```nix
{
  programs.anyrun = {
    enable = true;
    config = {
      plugins = [
        ...
        "${inputs.anyrun-plugins.packages.${pkgs.system}.example-plugin}/lib/libexample-plugin.so"
        ...
      ];
      ...
    };
  };
}
```

## Plugins

- [Bookmarks Launcher](./plugins/bookmarks-launcher/README.md)
  - Read from a bookmark file and launch in default browser.
