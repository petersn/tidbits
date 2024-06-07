#!/bin/sh
set -e
set -x
mkdir -p test/
cd test/
gogui-twogtp -black 'gnugo --mode gtp' -white 'python ../play.py' -auto -verbose -sgffile output -games 0 -alternate

#KATAGO_CMD='/home/snp/local/KataGo/cpp/katago gtp -config ../katago_gtp.cfg -model /home/snp/Downloads/kata1-b18c384nbt-s9996604416-d4316597426.bin.gz'
#gogui-twogtp -black "$KATAGO_CMD" -white 'python ../play.py' -auto -verbose -sgffile output -games 0 -alternate
