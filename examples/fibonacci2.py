def fib(n):
    if n < 2: return n
    return fib(n-1) + fib(n-2)

import sys
print(fib(int(sys.argv[1])))