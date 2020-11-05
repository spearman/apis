#!/bin/sh

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

cargo run --example interactive
make -f MakefileDot interactive

cargo run --example graphical
make -f MakefileDot graphical

exit
