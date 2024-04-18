#!/bin/bash

TMPFILE=stdout.cell
MAX=0

CMPXZ="xz -c -z -9"
CMPGZ="gzip -c -9 -f"

function div() { echo "scale=3 ; $1 / $2" | bc; }

function fsize() { stat -c %s $1; }

for i in $(seq 0 $1); do
	# 2**64 - 1
	#RULEN=$(shuf -i 1-18446744073709551615 -n 1)
	RULEN=$(shuf -i 1-7625597484987 -n 1)
	#RULEN=$i
	CMD="./target/release/ca1d 8 7 $RULEN @ --output=Raw --to=400 --width=400 --verbose=0"
	$CMD > $TMPFILE
	SZ=$($CMPXZ $TMPFILE | wc -c)

	if (( ${SZ} >= ${MAX} )); then
		echo -n $(div $SZ $(fsize $TMPFILE))
		echo -e " : ${CMD}"
		MAX=${SZ}
	fi

done

rm -f ${TMPFILE}
