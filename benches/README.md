# Tango benchmarks

These are benchmarks using [tango](https://github.com/bazhenov/tango), a benchmark tool supporting benchmark executions
of arbitrary programs or versions.

Purpose is to reliably compare the performance of different implementations in Gitopolis.

How to run:

```shell
# compare the current branch to main
gop benchmark-compare

# compare a target git-revision against a reference git-revision
gop benchmark-compare <target> <reference>
```
