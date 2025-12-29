import json
import pathlib
import sys


def load_labels(criterion_dir: str, group: str) -> dict[str, str]:
    patterns = [
        f"{group}/*/*/report/mean.gnuplot",
        f"{group}/*/*/report/median.gnuplot",
        f"{group}/*/*/report/typical.gnuplot",
    ]
    for pattern in patterns:
        for path in pathlib.Path(criterion_dir).glob(pattern):
            labels = {}
            try:
                text = path.read_text(encoding="utf-8", errors="ignore")
            except OSError:
                continue
            for line in text.splitlines():
                stripped = line.strip()
                if stripped.startswith("set title "):
                    labels["title"] = stripped[len("set title ") :].strip().strip("'\"")
                elif stripped.startswith("set xlabel "):
                    labels["xlabel"] = stripped[len("set xlabel ") :].strip().strip("'\"")
                elif stripped.startswith("set ylabel "):
                    labels["ylabel"] = stripped[len("set ylabel ") :].strip().strip("'\"")
            if labels:
                return labels
    return {}


def main() -> int:
    if len(sys.argv) != 4:
        raise SystemExit("usage: criterion_plot.py <group> <criterion_dir> <out_dir>")

    group = sys.argv[1]
    criterion_dir = sys.argv[2]
    out_dir = sys.argv[3]

    paths = list(pathlib.Path(criterion_dir).glob(f"{group}/*/*/new/estimates.json"))
    if not paths:
        raise SystemExit(
            f"No Criterion data for group '{group}'. Run `make bench` first."
        )

    data = {}
    variants = set()
    for path in paths:
        parts = path.parts
        try:
            idx = parts.index(group)
        except ValueError:
            continue
        if idx + 2 >= len(parts):
            continue
        variant = parts[idx + 1]
        count_str = parts[idx + 2]
        try:
            count = int(count_str)
        except ValueError:
            continue
        with path.open("r", encoding="utf-8") as handle:
            estimates = json.load(handle)
        mean = estimates["mean"]["point_estimate"]
        data.setdefault(count, {})[variant] = mean
        variants.add(variant)

    variants = sorted(variants)
    if not variants:
        raise SystemExit(f"No variants found for group '{group}'.")

    dat_path = pathlib.Path(out_dir) / f"{group}.dat"
    labels = load_labels(criterion_dir, group)
    with dat_path.open("w", encoding="utf-8") as handle:
        handle.write("count " + " ".join(variants) + "\n")
        for count in sorted(data):
            row = [str(count)]
            for variant in variants:
                value = data[count].get(variant)
                row.append("NaN" if value is None else f"{value:.6f}")
            handle.write(" ".join(row) + "\n")

    gnuplot_path = pathlib.Path(out_dir) / f"{group}.gnuplot"
    with gnuplot_path.open("w", encoding="utf-8") as handle:
        handle.write("set terminal svg size 1200,800 font 'Helvetica,12'\n")
        handle.write(f"set output '{dat_path.with_suffix('.svg')}'\n")
        title = labels.get("title", f"{group} mean time")
        xlabel = labels.get("xlabel", "Preload count")
        ylabel = labels.get("ylabel", "Mean time (ns)")
        handle.write(f"set title '{title}'\n")
        handle.write(f"set xlabel '{xlabel}'\n")
        handle.write(f"set ylabel '{ylabel}'\n")
        handle.write("set grid ytics\n")
        handle.write("set key left top\n")
        handle.write("set yrange [0:*]\n")
        handle.write("set datafile separator whitespace\n")
        handle.write(
            f"plot for [col=2:{len(variants)+1}] "
            f"'{dat_path}' using 1:col with linespoints title columnhead\n"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
