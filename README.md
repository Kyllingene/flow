# Flow Foo

## Starting

`flow level-file.foo`

## Level Syntax

Flow Foo only has 8 colors.

Each line should describe a pair of sources, like so:
`x1 y1 x2 y2`
Or a wall, like so:
`x y`

Except for the first line, which defines the board size:
`x_size y_size`

Example:
```
6 6

0 0 0 5
5 0 2 3
1 4 5 5
3 4 5 4
5 1 3 3
3 1 4 2

2 1
2 2
3 2
2 5

```

Creates the board:
```
A....B
..&F.E
..&F.
..BE..
.C&D.D
A....C
```

"Invalid" levels, i.e. ones you can solve without filling the board, or that have multiple solutions, are legal. When a flow touches both of it's sources, it is considered complete, but can continue going.

## Controls

- WASD / Arrow Keys : Move
- Space : Toggle color grab
- Escape / Q : Exit

Dragging a flow over another flow deletes the other flow. Dragging a flow over itself deletes itself(and toggles color grab off). Dragging a flow over any source (regardless of color) also toggles color grab off.

Once all the flows have been completed, the game ends. Your cursor will be adjacent to a source, not on top of it.