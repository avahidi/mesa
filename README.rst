MESA
====

*mesa* is a command line utility recording and comparing execution times.

You can use it for tracking performance improvements/regressions in an application over time.


Usage
-----

Assume we want to measure execution time for an application we are developing.
For this example I will use "sleep 1":

.. code-block:: console

    $ mesa -- sleep 1
     Age      Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     -------- ---------- --------- ---- -------- ---------- ----------
     just now      sleep         1    1   1.0029     0.0000

Many changes later, we want to see if performance has improved:

.. code-block:: console

    $ mesa -- sleep 1
     Age         Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     ----------- ---------- --------- ---- -------- ---------- ----------
        just now      sleep         1    1   1.0027     0.0000
     0:02:36 ago      sleep         1    1   1.0029     0.0000       0.02

Look at that, a 0.02% improvement :) 

Or maybe its just noise? Let us instead do average of multiple runs:

.. code-block:: console

    $ mesa --runs=10 -- sleep 1
     Age         Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     ----------- ---------- --------- ---- -------- ---------- ----------
        just now      sleep         1   10   1.0030     0.0003
     0:01:36 ago      sleep         1    1   1.0027     0.0000      -0.04
     0:04:12 ago      sleep         1    1   1.0029     0.0000      -0.02

Oh, no! Now performance has degraded with 0.04% instead :(


License
-------

This utility is licensed under GNU general public license version 2.
