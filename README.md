# Dungeon Keeper

A Tower Defense, Dungeon Management kind of game, but also inspired by Dwarf Fortress? Even I don't know where this game will go over time.

> Version 0.2.0.7_@wr0.5

## Changelog

- Switched to new `GlobalWorkID` instead of `UUID` for work identification.
- Added feature to remove works from `GlobalWorkValidator`.
- Works are registered and removed via events.
- Every pawn holds the data of every work in the world, instead of having works in a global pool.
- Switched from `GlobalWorkPool` to `GlobalWorkValidator`, which doesn't holds the works itself but holds their occupied status. `GlobalWorkValidator` is only used to determine whether a work exists and whether its occupied or not.
- Changed to `order::mine_order` and `turn::pawn_act_turn` functions.
- Fixed bug which caused works being stuck in the inaccessible list of workers.
- Fixed bug which caused workers to accept non-existent works.
- Removed unused use statements and variables.

## TO-DO

- Handle work register and remove events in batches, rather than handling them in a single frame.