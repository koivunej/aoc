#!/usr/bin/env bash

set -eu

run_day () {
	local day="$(printf '%02d' "$1")"
	if ! [ -f "input$day" ]; then
		if $2; then
			echo "day$day: input file missing" >&2;
			exit 1;
		fi
		return 0;
	fi

	cargo --quiet run --release --bin "day$day" < "input$day" | sed -Ee "s/^/day$day: /"
}

case $# in
	0) # run all days
		seq 1 25 | while read day; do run_day "$day" false; done
		;;
	1) # run specific day
		run_day "$1" true
		;;
	*)
		echo "Invalid arguments, usage: $0 [day]" >&2;
		exit 1;
		;;
esac
