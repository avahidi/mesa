
memo = {}
def fib(n):
    if n < 2: return n
    if n in memo: return memo[n]
    result = fib(n-1) + fib(n-2)
    memo[n] = result
    return result

import sys
print(fib(int(sys.argv[1])))