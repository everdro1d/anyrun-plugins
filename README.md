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
      inputs.nixpkgs.follows = "nixpkgs";
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
        "${inputs.anyrun-plugins.packages.${pkgs.system}.example-plugin"
        ...
      ];
      ...
    };
  };
}
```

## Plugins

- [Example](./plugins/example/README.md)
  - example part 2
