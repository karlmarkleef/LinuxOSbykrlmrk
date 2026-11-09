# Windows compatibility

Mark Linux runs Windows applications through upstream Wine and Proton based
runtimes. The Mark Compatibility Platform (MCP) layers integration, prefix and
hardware bridges on top of those upstream components instead of reimplementing
the Windows API.

## Modules

| Module | Source | Status |
|--------|--------|--------|
| Runtime manager | `windows/runtime-manager/` (`mark-runtime`) | Implemented |
| Prefix manager | `windows/prefix-manager/` (`mark-prefix`) | Implemented |
| PE inspector | `windows/pe-inspector/` (`mark-pe-inspect`) | Implemented |
| Wine integration | upstream `wine` / `wine-staging` via `mark-win-launch` | Integrated |
| Proton integration | upstream Proton via `mark-win-launch` | Integrated |
| Dependency manager | upstream Winetricks / Bottles integration | Planned |
| Application launcher | desktop overlay `mark-win-launch` | Implemented |
| Window bridge | Hyprland window rules for Windows apps | Planned |
| Filesystem bridge | Wine prefix drive mapping | Planned |
| Hardware bridge | Linux procfs/sysfs reporting via `mark-hardware` | Implemented |
| Compatibility database | `windows/compatibility-database/` (`games.json`) | Implemented |

## Launch flow

`mark-win-launch` performs the real integration steps:

1. Inspects the PE32/PE32+ header with `mark-pe-inspect`.
2. Selects or creates a Wine prefix with `mark-prefix`.
3. Exports Wine performance and graphics environment variables
   (`WINEARCH`, `WINE_LARGE_ADDRESS_AWARE`, `WINEESYNC`, `WINEFSYNC`, DXVK state
   cache, shader cache paths).
4. Launches through Wine, Wine Staging or Proton with optional GameMode.

The Mark Compatibility Center (`mark-compat-center`) exposes the same flows
through a Zenity UI and reports runtime, prefix and PE state.

## Current limits

- Gaming graphics layers (DXVK, VKD3R-Proton, Vulkan) are configured through the
  Wine/Proton environment; per-game shader cache and pipeline cache behaviour
  still depends on the upstream runtime.
- Anti-cheat bypass is out of scope; unsupported titles are reported as
  `Unsupported` or `Requires Official Developer Support`.
- Performance claims are never made without a recorded benchmark on the same
  hardware and configuration.
