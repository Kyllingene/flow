# Flow Foo

## Starting

`flow level-file.foo`

## Level Syntax

Flow Foo only supports 6x6 boards, and only has 8 colors.

Each line should describe a pair of sources, like so:
`y1 x1 y2 x2`

Example:
```
0 5 1 1
0 4 5 0
0 3 3 3
```

Creates the board:
```
...CBA
.A....
......
...C..
......
B.....
```

"Invalid" levels, i.e. ones you can solve without filling the board, or that have multiple solutions, are legal. When a flow touches both of it's sources, it is considered complete, but can continue going.

## Controls

- WASD / Arrow Keys : Move
- Space : Toggle color grab
- Escape / Q : Exit

Dragging a flow over another flow deletes the other flow. Dragging a flow over itself deletes itself(and toggles color grab off). Dragging a flow over any source (regardless of color) also toggles color grab off.

Once all the flows have been completed, the game ends. Your cursor will be adjacent to a source, not on top of it.