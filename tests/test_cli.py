"""CU-10 and CU-11. The command line tool and its discipline."""

import hashlib
import pathlib
import subprocess
import sys

from tests.test_catalogue import MINI


def run_cli(*args):
    return subprocess.run(
        [sys.executable, "-m", "cuesheet", *args],
        capture_output=True,
        text=True,
    )


def write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text)
    return str(path)


CONTEXT = (
    "context mini p\n"
    "fact tools = read, edit\n"
    "fact window = inside\n"
    "fact last = read\n"
)


def test_the_sheet_command_prints_the_sheet_and_exits_zero(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(tmp_path, "p.context", CONTEXT)
    result = run_cli("sheet", catalogue, context)
    assert result.returncode == 0
    assert result.stdout.startswith("sheet mini p\n")
    assert "cue alpha\n" in result.stdout
    assert "cue beta\n" in result.stdout
    assert result.stderr == ""


def test_a_malformed_input_exits_two_with_one_error_line(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(tmp_path, "p.context", "context mini\n")
    result = run_cli("sheet", catalogue, context)
    assert result.returncode == 2
    assert result.stdout == ""
    lines = [ln for ln in result.stderr.splitlines() if ln.strip()]
    assert len(lines) == 1
    assert "Traceback" not in result.stderr


def test_an_unreadable_input_exits_two_with_one_error_line(tmp_path):
    context = write(tmp_path, "p.context", CONTEXT)
    result = run_cli("sheet", str(tmp_path / "absent.catalogue"), context)
    assert result.returncode == 2
    lines = [ln for ln in result.stderr.splitlines() if ln.strip()]
    assert len(lines) == 1
    assert "Traceback" not in result.stderr


def test_generating_a_sheet_changes_neither_input(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(tmp_path, "p.context", CONTEXT)
    before = {
        p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
        for p in (catalogue, context)
    }
    result = run_cli("sheet", catalogue, context)
    assert result.returncode == 0
    after = {
        p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
        for p in (catalogue, context)
    }
    assert before == after
