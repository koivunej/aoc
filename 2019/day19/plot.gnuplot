set terminal pngcairo size 4096,4096 enhanced font 'Verdana,10'
set output 'part2.png'

set size square

plot "output" using (column(1)):(column(2) * -1) with dots, "output" using (column(3)):(column(4) * -1) with dots
