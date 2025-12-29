#!/usr/bin/env bash
set -euo pipefail

bench="${1:-filter_bench}"
criterion_dir="${2:-target/criterion}"
plot_dir="${3:-target/criterion/plots}"

tmp_home="$(mktemp -d)"
trap 'rm -rf "$tmp_home"' EXIT
printf 'set yrange [0:*]\n' > "$tmp_home/.gnuplot"

HOME="$tmp_home" cargo bench --bench "$bench"
mkdir -p "$plot_dir"

python3 tools/criterion_plot.py insert "$criterion_dir" "$plot_dir"
python3 tools/criterion_plot.py test "$criterion_dir" "$plot_dir"

python3 tools/criterion_fix_gnuplot.py "$criterion_dir"
find "$criterion_dir" -name '*.gnuplot' -print0 | xargs -0 -r -n 1 env HOME="$tmp_home" gnuplot

python3 tools/copy_lines_svg.py "$criterion_dir" "$plot_dir"
