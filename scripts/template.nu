#!/usr/bin/env nu

# Must be negative
const ERROR_ID = -1

# TODO: either count *all* packages, or only explicitly installed ones (e.g. either npm or npm-all)
def main [--show-erred(-e) --show-zeroes(-z)] {[
    # conlumn names
    [ id, count ];
    # rows
    [ asdf, {asdfc} ]
    [ cargo, {cargoc} ]
    [ npm, {npmc false} ]
    # [ npm-all, {npmc true} ]
    [ uv, {uvc} ]
  ] | par-each {
    update count { |pkg|
      try {
        do $pkg.count
      } catch {
        $ERROR_ID
      }
    }
  } | where { |pkg| (
    $show_erred and $pkg.count == $ERROR_ID or
    $show_zeroes and $pkg.count == 0 or
    $pkg.count > 0
  )} | each {
    let c: string = if $in.count == $ERROR_ID {
      '?'
    } else {
      $in.count | into string
    }

    $"($c) \(($in.id)\)"
  } | str join ', '
}

# Using GNU awk to count lines following certain patterns
# (I prefer gawk to sed, sorry)
def count_lines_not_starting_2_spaces [] {
  gawk 'BEGIN {c=0} /\w\w.+/ {c++} END {print c}' | into int
}

def count_lines [] {
  wc -l | into int
}

def asdfc [] {
  # asdfc list package names at the first column, the each executable indented by 2 spaces
  # so we can count the lines that do not start with 2 spaces
  asdf list e> /dev/null | gawk 'BEGIN {c=0} /\w\w.+/ {c++} END {print c}' | into int
}

def cargoc [] {
  # cargo list package names at the first column, the each executable indented by 2 spaces
  # so we can count the lines that do not start with 2 spaces
  cargo install --list e> /dev/null | count_lines_not_starting_2_spaces
}

def npmc [deps: bool] {
  # npm returns an extra line at the top that we don't want
  (npm list (if $deps {'-agp'} else {'-gp'}) e> /dev/null | count_lines) - 1
}

def uvc [] {
  # uv *seems* to return 2 lines for each package
  (uv tool list e> /dev/null | count_lines) // 2
}

