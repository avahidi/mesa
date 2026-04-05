mesa
====

*mesa* is a command line utility for recording and comparing execution times. Instead of manually measuring performance with ``time``, mesa automates measurements across multiple runs and tracks your changes in a human-readable database.

*mesa* is similar to `hyperfine`_, but focuses more on long-term tracking.

.. _hyperfine: https://github.com/sharkdp/hyperfine


Quick start
-----------

Run any command with mesa to measure its execution time:

.. code-block:: console


    $ mesa -- python3 fibonacci.py 30

       Age    |         Program         | Runs |  Mean  | StdDev
    ----------+-------------------------+------+--------+--------
     just now | python3 fibonacci.py 30 |  3   | 0.1660 | 0.0043



That's it. Mesa records the time, and you can run the same command again later to compare.


A slightly longer example: tracking Fibonacci improvements
----------------------------------------------------------

Suppose you have a slow Fibonacci implementation and want to optimise it systematically:

.. code-block:: python

    # fibonacci.py
    def fib(n):
        if n == 0: return 0
        if n == 1: return 1
        return fib(n-1) + fib(n-2)

    import sys
    print(fib(int(sys.argv[1])))

You could run ``time`` manually after each change, but this is tedious. And a single run can contain some noise. More importantly, how do you keep track of all these numbers?


The Solution
~~~~~~~~~~~~

Mesa solves this by automating measurements and averaging multiple runs. Start by establishing a baseline with your original code:


.. code-block:: console

    $ mesa --runs=10 --note="original code" -- python3 fibonacci1.py 30

       Age    |         Program          | Runs |  Mean  | StdDev |     Note
    ----------+--------------------------+------+--------+--------+---------------
     just now | python3 fibonacci1.py 30 |  10  | 0.1509 | 0.0190 | original code


Mesa ran the command 10 times, averaging the results to reduce noise. The measurements are now stored in ``fibonacci.mesa`` for future comparisons. Notice that both mean and standard deviation of execution time are recorded.


Iteration 1: a tiny improvement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Let's make a small optimisation:


.. code-block:: python

    def fib(n):
        if n < 2: return n
        return fib(n-1) + fib(n-2)

.. code-block:: console

    $ mesa --runs=10 --note="removed one if" -- python3 fibonacci2.py 30

          Age       |         Program          | Runs |  Mean  | StdDev | Change (%) |      Note
    ----------------+--------------------------+------+--------+--------+------------+----------------
        just now    | python3 fibonacci2.py 30 |  10  | 0.1423 | 0.0095 |            | removed one if
        0:00:25 ago | python3 fibonacci1.py 30 |  10  | 0.1509 | 0.0190 |   -6.06    | original code


Looks like that improved performance a bit. Note that this tiny improvement would probably have drowned in system noise had we used ``time``...

Iteration 2: adding memoization
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

According to Wikipedia, memoization is an optimization technique used primarily to cache expensive function calls. Let's give it a try:

.. code-block:: python

    memo = {}
    def fib(n):
        if n < 2: return n
        if n in memo: return memo[n]
        result = fib(n-1) + fib(n-2)
        memo[n] = result
        return result


.. code-block:: console

    $ mesa --runs=10 --note="memoization" -- python3 fibonacci3.py 30

          Age       |         Program          | Runs |  Mean  | StdDev | Change (%) |      Note
    ----------------+--------------------------+------+--------+--------+------------+----------------
        just now    | python3 fibonacci3.py 30 |  10  | 0.0291 | 0.0064 |            |  memoization
        0:00:35 ago | python3 fibonacci2.py 30 |  10  | 0.1423 | 0.0095 |  -389.47   | removed one if
        0:01:00 ago | python3 fibonacci1.py 30 |  10  | 0.1509 | 0.0190 |  -419.12   | original code


That is a massive improvement over both baseline and our previous version!
But can we do even better? Maybe by removing recursion altogether?

Iteration 3: eliminate recursion
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Ask any CS student and they will confidently tell you that the non-recursive version will be vastly faster than anything else. Let's give it a try:

.. code-block:: python

    def fib(n):
        a, b = 0, 1
        for _ in range(n):
            a, b = b, a + b
        return a

.. code-block:: console

    $ mesa --runs=10 --note="iterative" -- python3 fibonacci4.py 30

          Age       |         Program          | Runs |  Mean  | StdDev | Change (%) |      Note
    ----------------+--------------------------+------+--------+--------+------------+----------------
        just now    | python3 fibonacci4.py 30 |  10  | 0.0282 | 0.0072 |            |   iterative
        0:00:19 ago | python3 fibonacci3.py 30 |  10  | 0.0291 | 0.0064 |   -3.28    |  memoization
        0:00:54 ago | python3 fibonacci2.py 30 |  10  | 0.1423 | 0.0095 |  -405.52   | removed one if
        0:01:19 ago | python3 fibonacci1.py 30 |  10  | 0.1509 | 0.0190 |  -436.15   | original code


This was a much smaller improvement than anticipated. It turns out dictionaries in python are very efficient, which I did not know until this experiment.

This highlights the importance of empirical measurements instead of making assumptions.
And this is why I wrote mesa: to replace opinions and feelings with hard facts, in a format that can be stored in your git repository.


Beyond measuring time
---------------------

It is possible to measure data beyond execution time, if this data happens to be part of the program output and the text before and after the data is known.

.. code-block:: console

    $ python3 pathfinder.py bfs
    Using algorithm 'bfs' we visited 32 nodes

    $ mesa --capture="/visited/nodes/" -- python3 pathfinder.py bfs

       Age    |          Program          | Runs |  Mean   | StdDev
    ----------+---------------------------+------+---------+--------
     just now | python3 pathfinder.py bfs |  3   | 32.0000 | 0.0000


The capture specifier has the format ``/before/after/`` (just like *sed*, the marker / can be any character).

The *before* part can be a sequence of strings. Each one must be matched in order before capturing begins.
Consider for example the output ``My uncle is 50 years old but my other uncle is 55 years old``.
To capture the first number you can use ``--capture="/is/years/"`` but to capture the second number you should use ``--capture="/is/is/years/"``.

Database format
---------------

The measurements database is a simple plain-text file:

.. code-block:: text

    # mesa database|github.com/avahidi/mesa|version=1.2
    <timestamp>|<executable>|<arguments>|<runs>|<mean>|<stddev>|<note>
    <timestamp>|<executable>|<arguments>|<runs>|<mean>|<stddev>|<note>
    ...

This makes it easy to review changes in git and understand your performance history over time.


Building from Source
---------------------

*mesa* is written in Rust with no external dependencies:

.. code-block:: console

    git clone https://github.com/avahidi/mesa
    cd mesa
    cargo build
    cargo run -- -- sleep 1 # test it...


About the Name
--------------

The name *mesa* is a play on the word *measurement*, or possibly a play on the Swedish word *"mes"* (coward). Pick whichever suits you.

License
-------

Licensed under GNU General Public License version 2.
