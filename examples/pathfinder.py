
from collections import deque
import heapq
import sys
import random


maze = """
##########
S   #    #
# # # ## #
# #   #  #
# ##### ##
#        #
######## #
#        #
# ######E#
"""

# create the 2D grid
grid = []
for r, row in enumerate(maze.strip().split('\n')):
    grid.append([])
    for c, ch in enumerate(row):
        if ch == 'S': start = (r,c)
        if ch == 'E': end = (r,c)
        grid[-1].append(0 if ch == ' ' or ch in 'SE' else 1)

rows, cols = len(grid), len(grid[0])


def neighbors(pos):
    r,c = pos
    return [(r+dr,c+dc) for dr,dc in [(-1,0),(1,0),(0,-1),(0,1)]
            if 0<=r+dr<rows and 0<=c+dc<cols and grid[r+dr][c+dc]==0]

def bfs(s, e):
    visited, queue = {s}, deque([s])
    while queue:
        node = queue.popleft()
        if node == e: return len(visited)
        for n in neighbors(node):
            if n not in visited:
                visited.add(n); queue.append(n)

def dfs(s, e):
    visited, stack = set(), [s]
    while stack:
        node = stack.pop()
        if node in visited: continue
        visited.add(node)
        if node == e: return len(visited)
        stack.extend(neighbors(node))

def nb(pos):
    r,c = pos
    return [(r+dr,c+dc) for dr,dc in [(-1,0),(1,0),(0,-1),(0,1)]
            if 0<=r+dr<ROWS and 0<=c+dc<COLS and grid[r+dr][c+dc]==0]

def random_walk(s, e, max_steps=3000):
    node,visited = s,[]
    for _ in range(max_steps):
        visited.append(node)
        if node==e: return len(visited)
        ns=neighbors(node)
        if not ns: break
        node=random.choice(ns)
    return len(visited)

def astar(s, e):
    h = lambda p: abs(p[0]-e[0]) + abs(p[1]-e[1])
    visited, heap = set(), [(h(s), s)]
    while heap:
        _, node = heapq.heappop(heap)
        if node in visited: continue
        visited.add(node)
        if node == e: return len(visited)
        for n in neighbors(node): heapq.heappush(heap, (h(n), n))

algos = {
    "dfs": dfs,
    "bfs": bfs,
    "random_walk": random_walk,
    "astar": astar,
    "A*": astar
}

typ = (sys.argv[1:] + ["dfs"])[0]
f = algos.get(typ)

if f is None:
    print("Unknown algorithm ", typ)
    sys.exit(20)

v = f(start, end)
print( f"Using algorithm '{typ}' we visited {v} nodes")