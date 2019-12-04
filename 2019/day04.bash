#!/usr/bin/env bash

set -eu
set -o pipefail

echo -n 'stage1: '
seq "$1" "$2" \
	| egrep '^(1+)?(2+)?(3+)?(4+)?(5+)?(6+)?(7+)?(8+)?(9+)?$' \
	| egrep '(.)\1'

# echo -n 'stage2: '
# seq "$1" "$2" \
# 	| egrep '^(1+)?(2+)?(3+)?(4+)?(5+)?(6+)?(7+)?(8+)?(9+)?$' \
# 	| egrep '(.)\1' \
# 	| egrep -v '(.)\1\1'
