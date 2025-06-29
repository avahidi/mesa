MESA
====

*mesa* is a command line utility recording and comparing execution times.


Usage
-----

Assume we want to measure execution time for an application, such as "sleep 1":

.. code-block:: console

    $ mesa -- sleep 1
     Age      Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     -------- ---------- --------- ---- -------- ---------- ----------
     just now      sleep         1    1   1.0029     0.0000


If we repeat this again, latest run is compared to the first one:

.. code-block:: console

    $ mesa -- sleep 1
     Age         Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     ----------- ---------- --------- ---- -------- ---------- ----------
        just now      sleep         1    1   1.0027     0.0000
     0:02:36 ago      sleep         1    1   1.0029     0.0000       0.02


At this point maybe we realize a single run is too noisy, so lets run "sleep 1" 10 times:

.. code-block:: console

    $ mesa --runs= 10 -- sleep 1
     Age         Executable Arguments Runs Mean (s) StdDev (s) Change (%)
     ----------- ---------- --------- ---- -------- ---------- ----------
        just now      sleep         1   10   1.0030     0.0003
     0:01:36 ago      sleep         1    1   1.0027     0.0000      -0.04
     0:04:12 ago      sleep         1    1   1.0029     0.0000      -0.02


Well, now we are pretty sure we know has fast sleep is...

