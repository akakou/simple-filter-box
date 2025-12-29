import pathlib
import sys


def ensure_zero_yrange(lines: list[bytes]) -> tuple[list[bytes], bool]:
    for i, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith(b"set yrange"):
            line_ending = b"\n"
            if line.endswith(b"\r\n"):
                line_ending = b"\r\n"
            lines[i] = b"set yrange [0:*]" + line_ending
            return lines, True
    return lines, False


def split_header(data: bytes) -> tuple[bytes, bytes]:
    plot_idx = data.find(b"\nplot ")
    if plot_idx == -1:
        plot_idx = data.find(b"\r\nplot ")
    if plot_idx == -1:
        return data, b""

    line_end = data.find(b"\n", plot_idx + 1)
    if line_end == -1:
        return data, b""
    return data[: line_end + 1], data[line_end + 1 :]


def fix_file(path: pathlib.Path) -> bool:
    original = path.read_bytes()
    header, tail = split_header(original)
    lines = header.splitlines(keepends=True)
    updated_lines, changed = ensure_zero_yrange(lines)
    if not changed:
        return False
    new_header = b"".join(updated_lines)
    path.write_bytes(new_header + tail)
    return True


def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("usage: criterion_fix_gnuplot.py <report_dir>")

    report_dir = pathlib.Path(sys.argv[1])
    if not report_dir.exists():
        raise SystemExit(f"report dir not found: {report_dir}")

    changed = 0
    for path in report_dir.rglob("*.gnuplot"):
        if fix_file(path):
            changed += 1

    print(f"updated {changed} gnuplot files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
