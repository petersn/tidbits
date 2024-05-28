#!/bin/sh
set -e
set -x
gogui-twogtp -black 'gnugo --mode gtp' -white 'python ../play.py' -auto -verbose -sgffile output -games 0
