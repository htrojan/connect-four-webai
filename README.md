# connect-four-webai
A small connect-four ai written in rust/webassembly.
Currently hosted at [lobbi44.de](https://lobbi44.me).

Uses alpha-beta pruning to estimate the best move of the computer.
Depending on the number of stones on the field, two variations are used:
- If only a few stones are on the field and the game tree can not be build up to the end of the game, the tree depth is limited to a a few plies and a heuristic is used that counts the number of open chains of three for each player
- As soon as it becomes computationally feasible, the heuristic is switched to a simple win/loose/draw metric, a score only assuming the values -1, 0, 1. This brings down the computational cost of the heuristic function and makes a larger search-depth possible
