# z_life2

Cellular automata used to generate a terrain as well as perpetuate mutating life cells that operate on a certain ruleset.
Built mainly using the Pixels and game-loop crate.

Below is an example

https://github.com/nikkolaka/z_life2/assets/88300050/0b3115a4-1480-4628-ba17-48688fe7ac72

# Controls:
  - **Left Arrow** : Generates a new random noise seed with the set wall density and zoom (Warning: will delete any living cells)
  - **Up Arrow** : Generates a new seed with an increased density in walls
  - **Down Arrow** : Generates a new seed with an increased density in walls
  - **Right Arrow** : Smooths out noise to create a more realistic cave type map
  - **Z** : creates larger noise pixels
  - **X** : creates smaller noise pixels
  - **Escape** : Closes program

# WIP:
  - Make mutations rely on average of life cells around it
  - Make more params of life cells controllable by user
  - Smoothing of larger pixels should be more organic
  - Text on screen (?)

