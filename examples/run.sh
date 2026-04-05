#!/usr/bin/env bash

set -e


cd .. && cargo build && cd examples # cargo -C <dir> is still unstable
rm -f *.mesa

# 1
echo "Example 1: measuring time..."

export CMD="../target/debug/mesa --runs=10 --database=fibonacci.mesa"
$CMD --note="original code" --output=""  -- python3 fibonacci1.py 30
$CMD --note="removed one if" --output=""  -- python3 fibonacci2.py 30
$CMD --note="memoization" --output=""  -- python3 fibonacci3.py 30
$CMD --note="iterative" --output="stdout.table"  -- python3 fibonacci4.py 30

# 2
echo "Example 2: program data capture..."

export CMD="../target/debug/mesa --runs=1 --database=fibonacci_nodes.mesa"
$CMD --capture=/visited/nodes/ --output=""  --note="Random Walk" -- python3 pathfinder.py random_walk
$CMD --capture=/visited/nodes/ --output=""  --note="BFS" -- python3 pathfinder.py bfs
$CMD --capture=/visited/nodes/ --output=""  --note="DFS" -- python3 pathfinder.py dfs
$CMD --capture=/visited/nodes/ --output="stdout.table"  --note="A*" -- python3 pathfinder.py astar

# 3
echo "Example 3: program data capture, higher is better..."

export CMD="../target/debug/mesa --runs=1 --database=fibonacci_compress.mesa"
$CMD --capture="/ratio of//" --reverse --output="" --note="Baseline" -- python3 compress.py baseline
$CMD --capture="/ratio of//" --reverse --output="" --note="zlib" -- python3 compress.py zlib
$CMD --capture="/ratio of//" --reverse --output="" --note="bz2" -- python3 compress.py bz2
$CMD --capture="/ratio of//" --reverse --output="stdout.table" --note="lzma" -- python3 compress.py lzma
