#!/bin/sh
set -e
set -x
mkdir -p test/
cd test/
gogui-twogtp -black 'gnugo --mode gtp' -white 'python ../play.py' -auto -verbose -sgffile output -games 0 -alternate
