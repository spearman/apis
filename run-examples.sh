#!/bin/bash

set -e

while getopts "v" opt; do
  case $opt in
    v)
      set -x
      ;;
    *)
      echo "Valid options:"
      echo "  -v    verbose"
      exit 1
      ;;
  esac
done

for e in `ls examples/ -I readme.rs`; do
  example_name=`basename -s .rs $e`
  cargo run --example $example_name
  make -f MakefileDot $example_name
done

cargo run --example readme
make -f MakefileDot charsink
make -f MakefileDot intsource
make -f MakefileDot myprogram

exit
