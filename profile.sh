#!/usr/bin/sh

# usage: ./profile.sh <binary> [args...] > onoro.svg

set -e

cargo b --profile profiled
rm -f perf.data
perf record -g -F 999 --call-graph dwarf -- $@ >/dev/null
perf script -F comm,pid,tid,time,event,ip,sym,dso,trace | stackcollapse-perf.pl | flamegraph.pl > brc.svg
