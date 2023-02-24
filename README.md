# Dungeon Keeper

A Tower Defense, Dungeon Management kind of game, but also inspired by Dwarf Fortress? Even I don't know where this game will go over time.

> Version 0.2.0.2

## Changelog

- World now has a copy of each tile's data in a tile grid.
- Implemented the global work pool.

## TODO

- Behaviour
  - A pice of code which determines what a pawn does under certain conditions, behaviours basically sends taks to that pawn to execute.
  - A pawn should be able to support multiple behaviours unless they fully contradict one another.
  - Behaviours can't override player given tasks, or player given task priorties.