import pathlib
import shutil
import sys


def output_name(path: pathlib.Path) -> str:
    try:
        report_index = path.parts.index("report")
    except ValueError:
        report_index = len(path.parts)
    name_parts = [part for part in path.parts[:report_index] if part]
    if not name_parts:
        name_parts = ["lines"]
    return "_".join(name_parts) + "_lines.svg"


def main() -> int:
    if len(sys.argv) != 3:
        raise SystemExit("usage: copy_lines_svg.py <criterion_dir> <plot_dir>")

    criterion_dir = pathlib.Path(sys.argv[1])
    plot_dir = pathlib.Path(sys.argv[2])
    plot_dir.mkdir(parents=True, exist_ok=True)

    for path in criterion_dir.rglob("report/lines.svg"):
        rel = path.relative_to(criterion_dir)
        out_name = output_name(rel)
        shutil.copy2(path, plot_dir / out_name)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
