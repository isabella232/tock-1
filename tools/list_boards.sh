#!/usr/bin/env bash

# Find boards based on folders with Makefiles
boards="helium-feather/"
# for b in $(find boards | egrep 'Makefile$'); do
#     b1=${b#boards/}
#     b2=${b1%/*}
#     boards+="$b2 "
# done
# echo "${boards}"

for board in $boards; do
    echo $board
done
