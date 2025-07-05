MESA
====

*mesa* is a command line utility for recording and comparing execution times. You can use it to track performance of applications you are developing over time.

Usage
-----

To measure the execution time of a command, run it with `mesa`:

.. code-block:: console

    $ mesa [mesa flags] -- <command plus arguments>

The `--` is used to separate `mesa`'s own arguments from the arguments for the command being executed.

Example: improving Fibonacci
----------------------------

Assume we have an implementation of Fibonacci in python:

.. code-block:: python

    # fibonacci.py
    def fib(n):
        if n == 0: return 0
        if n == 1: return 1
        return fib(n-1) + fib(n-2)

    import sys
    print(fib(int(sys.argv[1])))

We would like to improve this code in a systematic manner, and document if our changes lead to an actual performance improvement.
One way to do this is to run the UNIX utility *time* manually after every change:

.. code-block:: console

    $ time python3 fibonacci.py 30
    832040

    real    0m0.183s
    user    0m0.174s
    sys     0m0.010s

But this gets cumbersome very quickly. Furthermore a single run can contain a lot of system noise.

Let us address both issues by using *mesa* to automate this process and also record the average of multiple runs.
We start by recording execution time for our original code to establish a baseline:


.. code-block:: console

    $ mesa --runs=10 --database=fibonacci.perf --note="original code" -- python3 fibonacci.py 30

       Age    | Executable |    Arguments    | Runs | Mean (s) | StdDev (s) | Change (%) |     Note
    ----------+------------+-----------------+------+----------+------------+------------+---------------
     just now |  python3   | fibonacci.py 30 |  10  |  0.1497  |   0.0232   |            | original code

Note that we only measure wall clock time. Next, assume we made some changes and want to see if those improve performance:


.. code-block:: python

    def fib(n):
        if n < 2: return n
        return fib(n-1) + fib(n-2)

.. code-block:: console

    $ mesa --runs=10 --database=fibonacci.perf --note="removed one if" -- python3 fibonacci.py 30

          Age       | Executable |    Arguments    | Runs | Mean (s) | StdDev (s) | Change (%) |      Note
    ----------------+------------+-----------------+------+----------+------------+------------+----------------
        just now    |  python3   | fibonacci.py 30 |  10  |  0.1442  |   0.0186   |            | removed one if
        0:01:54 ago |  python3   | fibonacci.py 30 |  10  |  0.1497  |   0.0232   |    3.82    | original code

Surprisingly, that actually improved performance a tiny bit. Our next improvement idea is adding an ad-hoc cache (aka.`memoization`):


.. code-block:: python

    memo = {}
    def fib(n):
        if n < 2: return n
        if n in memo: return memo[n]
        result = fib(n-1) + fib(n-2)
        memo[n] = result
        return result


.. code-block:: console

    $ mesa --runs=10 --database=fibonacci.perf --note="memoization" -- python3 fibonacci.py 30

          Age       | Executable |    Arguments    | Runs | Mean (s) | StdDev (s) | Change (%) |      Note
    ----------------+------------+-----------------+------+----------+------------+------------+----------------
        just now    |  python3   | fibonacci.py 30 |  10  |  0.0369  |   0.0007   |            |  memoization
        0:01:25 ago |  python3   | fibonacci.py 30 |  10  |  0.1442  |   0.0186   |   290.57   | removed one if
        0:03:19 ago |  python3   | fibonacci.py 30 |  10  |  0.1497  |   0.0232   |   305.47   | original code


This was a clear improvement, but surely the  optimal solution would be to eliminate recursive calls altogether?

.. code-block:: python

    def fib(n):
        a, b = 0, 1
        for _ in range(n):
            a, b = b, a + b
        return a

.. code-block:: console

    $ mesa --runs=10 --database=fibonacci.perf --note="not recursive" -- python3 fibonacci.py 30

          Age       | Executable |    Arguments    | Runs | Mean (s) | StdDev (s) | Change (%) |      Note
    ----------------+------------+-----------------+------+----------+------------+------------+----------------
        just now    |  python3   | fibonacci.py 30 |  10  |  0.0361  |   0.0005   |            | not recursive
        0:12:35 ago |  python3   | fibonacci.py 30 |  10  |  0.0369  |   0.0007   |    2.24    |  memoization
        0:14:00 ago |  python3   | fibonacci.py 30 |  10  |  0.1442  |   0.0186   |   299.32   | removed one if
        0:15:54 ago |  python3   | fibonacci.py 30 |  10  |  0.1497  |   0.0232   |   314.56   | original code


This was a smaller improvement than anticipated, highlighting the importance of empirical measurement. This is exactly why I wrote *mesa*: to replace opinions and feelings with hard facts that can be tracked in a git repository.

Measurement database
--------------------

The mesa database is the file where measurements are stored, and developers might want to include it in their version control system. By default this file is called "timing.mesa" in the current folder, although that can be changed using the `--database` option.

The v1.2 file format looks like this:

.. code-block::

    # mesa database|github.com/avahidi/mesa|version=1.2
    <timestamp>|<executable>|<arguments>|<runs>|<mean>|<stddev>|<note>
    <timestamp>|<executable>|<arguments>|<runs>|<mean>|<stddev>|<note>
    ...

It is assumed that the entries are sorted by timestamp.

Output format
-------------

By default output is written to the console as a table. This can however be changed by specifying `--output=<filename.ext>`, where the extension decides the output format (accepted extensions are: txt, xml, csv and json).

The filename *stdout* is assumed to mean the standard output and not a file. For example, this will dump JSON to stdout instead of creating the file stdout.json:

.. code-block:: console

    mesa --output=stdout.json ...

Although why anyone would want to do something like that is beyond me.


Development
-----------

*mesa* was developed in Rust and does not use any external libraries.

To build it from source and run it directly from cargo try this:

.. code-block:: console

    git clone https://github.com/avahidi/mesa
    cd mesa
    cargo build
    cargo run -- -- sleep 1

Misc.
-----

The name *mesa* is either a play on the word *measurement* or the Swedish word *"mes"* (*coward*). Pick whichever suits you.

License
-------

This utility is licensed under GNU General Public License version 2.
