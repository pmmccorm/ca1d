#!/bin/bash

set -euo pipefail

RADIX=$1
NABORS=$2
START=${3:-'@'}

ITERS=10000

# because bc breaks its lines ?!
export BC_LINE_LENGTH=0

TMPFILE=stdout.cell
MAX=0

CMPXZ="xz -c -z -9"
CMPGZ="gzip -c -9 -f"
CMPZSTD="zstd -c -9 -f"

function div() { echo "scale=3 ; $1 / $2" | bc; }

function fsize() { stat -c %s $1; }

for i in $(seq 0 $ITERS); do
	RULE_COUNT=$(echo "$RADIX^$RADIX^$NABORS" | bc)
	RULE_NUMBER=$(python3 -S -c "import random; print (random.randrange(1,$RULE_COUNT))")
	CMD="./target/release/ca1d $RADIX $NABORS $RULE_NUMBER $START --output=Raw --to=200 --width=200 --verbose=0"
	$CMD > $TMPFILE
	SZ=$($CMPZSTD $TMPFILE | wc -c)

	if (( ${SZ} >= ${MAX} )); then
		echo -n $(div $SZ $(fsize $TMPFILE))
		echo -e " : ${CMD}"
		MAX=${SZ}
	fi

done

rm -f ${TMPFILE}
