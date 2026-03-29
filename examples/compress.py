import zlib, bz2, lzma, sys

# some test data to compress
text = (b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur." * 100 +
        b"The quick brown fox jumps over the lazy dog " * 100)

def baseline(x):
    return x

algos = {
    "baseline": baseline,
    "zlib": zlib.compress,
    "bz2": bz2.compress,
    "lzma": lzma.compress,
}

typ = (sys.argv[1:] + ["baseline"])[0]
f = algos.get(typ)

if f is None:
    print("Unknown compression type ", typ)
    sys.exit(20)

compressed = f(text)
ratio = len(text) / len(compressed)
print( f"The algorithm '{typ}' gave a compression ratio of {ratio}")
