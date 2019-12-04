#!/usr/bin/env bash

set -eu
set -o pipefail

echo -n 'stage1: '
seq "$1" "$2" \
	| egrep '^1*2*3*4*5*6*7*8*9*$' \
	| egrep '(.)\1' \
	| wc -l

echo -n 'stage2: '
seq "$1" "$2" \
	| egrep '^1*2*3*4*5*6*7*8*9*$' \
	| egrep '(.)\1' \
	| while read line; do \
		grep -o . <<< "$line" \
			| uniq -c \
			| egrep -q '^\s+2\s' \
			&& echo "$line"
	done \
	| wc -l
