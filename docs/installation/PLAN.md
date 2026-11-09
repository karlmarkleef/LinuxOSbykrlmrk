# Installation direction

The target is a GUI installer running in the live Mark Linux session. Planned stages are welcome, locale/keyboard, network, hardware detection, installation mode, exact disk/partition review, user account, performance profile, summary, progress and finish.

Disk operations must show the exact target device and partitions and require explicit confirmation. Prefer tested partitioning tools. Installer tests must use disposable VM disks; never use the development host physical disks for destructive tests.

No installer code exists in Phase 0.