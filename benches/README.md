# Tango benchmarks

These are benchmarks using [tango](https://github.com/bazhenov/tango), a benchmark tool supporting benchmark executions
of arbitrary programs or versions.

Purpose is to reliably compare the performance of different implementations in Vaquera.

How to run:

```shell
# compare the current branch to main
vaq benchmark-compare

# compare a target git-revision against a reference git-revision
vaq benchmark-compare <target> <reference>
```
