"""CU-14 to CU-16. The gaps view and the partition of the two views."""

import pathlib
import subprocess
import sys

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.sheet import build_gaps, build_sheet, format_gaps
from tests.test_catalogue import MINI

CATALOGUE = parse_catalogue(MINI, "mini.catalogue")
EXAMPLES = pathlib.Path(__file__).resolve().parents[1] / "examples"

DELTA = MINI + """action delta
requires tools lacks edit
fires delta fires text
fires-in-turn delta turn text
"""

EPSILON = MINI + """action epsilon
requires window is-not inside
fires epsilon fires text
fires-in-turn epsilon turn text
"""


def divide(catalogue, context_text):
    context = parse_context(context_text, "c.context", catalogue)
    sheet = [cue.name for cue in build_sheet(catalogue, context)]
    gaps = [gap.name for gap in build_gaps(catalogue, context)]
    return context, sheet, gaps


def gaps_text(catalogue, context_text):
    context = parse_context(context_text, "c.context", catalogue)
    return format_gaps(context, build_gaps(catalogue, context))


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


def test_a_gap_block_names_only_the_tests_that_did_not_pass():
    text = gaps_text(
        CATALOGUE,
        "context mini p\n"
        "fact tools = read, edit\n"
        "fact window = outside\n"
        "fact last = read\n",
    )
    assert text == (
        "gaps mini p\n"
        "gap beta\n"
        "blocked window is inside, recorded outside\n"
    )


def test_every_failing_test_is_accounted_for_in_catalogue_order():
    text = gaps_text(
        CATALOGUE,
        "context mini p\n"
        "fact tools = read\n"
        "fact window = outside\n"
        "fact last = read\n",
    )
    assert text == (
        "gaps mini p\n"
        "gap beta\n"
        "blocked tools has edit, recorded read\n"
        "blocked window is inside, recorded outside\n"
    )


def test_a_test_on_an_unrecorded_fact_takes_the_not_recorded_form():
    text = gaps_text(
        CATALOGUE,
        "context mini p\nfact tools = read, edit\nfact last = read\n",
    )
    assert text == (
        "gaps mini p\n"
        "gap beta\n"
        "blocked window is inside, not recorded\n"
    )


def test_blocked_lines_cover_the_is_not_and_lacks_tests():
    catalogue = parse_catalogue(DELTA, "delta.catalogue")
    text = gaps_text(
        catalogue,
        "context mini p\n"
        "fact tools = read, edit\n"
        "fact window = inside\n"
        "fact last = read\n",
    )
    assert "gap delta\n" in text
    assert "blocked tools lacks edit, recorded edit, read\n" in text
    catalogue = parse_catalogue(EPSILON, "epsilon.catalogue")
    text = gaps_text(
        catalogue,
        "context mini p\nfact window = inside\n",
    )
    assert "gap epsilon\n" in text
    assert "blocked window is-not inside, recorded inside\n" in text


def test_the_two_views_partition_the_catalogue():
    contexts = [
        "context mini p\nfact tools = read, edit\n"
        "fact window = inside\nfact last = read\n",
        "context mini p\nfact tools = read\n"
        "fact window = outside\nfact last = read\n",
        "context mini p\nfact tools = edit\nfact last = none\n",
        "context mini p\nfact last = none\n",
    ]
    for context_text in contexts:
        _, sheet, gaps = divide(CATALOGUE, context_text)
        assert sorted(sheet + gaps) == sorted(CATALOGUE.actions)
        assert not set(sheet) & set(gaps)


def test_the_two_views_partition_the_example_contexts():
    catalogue = parse_catalogue(
        (EXAMPLES / "stagehand.catalogue").read_text(), "stagehand.catalogue"
    )
    for context_name, sheet_names, gap_names in (
        (
            "ordinary.context",
            [
                "ask-human",
                "edit-file",
                "read-file",
                "run-command",
                "search-corpus",
                "send-update",
            ],
            ["spawn-helper"],
        ),
        (
            "restricted.context",
            ["read-file", "search-corpus"],
            [
                "ask-human",
                "edit-file",
                "run-command",
                "send-update",
                "spawn-helper",
            ],
        ),
    ):
        context = parse_context(
            (EXAMPLES / context_name).read_text(), context_name, catalogue
        )
        assert [cue.name for cue in build_sheet(catalogue, context)] == sheet_names
        assert [gap.name for gap in build_gaps(catalogue, context)] == gap_names


def test_no_gap_block_ever_stands_empty():
    catalogue = parse_catalogue(
        (EXAMPLES / "stagehand.catalogue").read_text(), "stagehand.catalogue"
    )
    for context_name in ("ordinary.context", "restricted.context"):
        context = parse_context(
            (EXAMPLES / context_name).read_text(), context_name, catalogue
        )
        for gap in build_gaps(catalogue, context):
            assert gap.blocked


def test_when_nothing_is_unavailable_the_gaps_view_is_its_header_alone():
    text = gaps_text(
        CATALOGUE,
        "context mini p\n"
        "fact tools = read, edit\n"
        "fact window = inside\n"
        "fact last = read\n",
    )
    assert text == "gaps mini p\n"


def test_the_gaps_command_prints_and_exits_zero(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(
        tmp_path,
        "p.context",
        "context mini p\nfact tools = read\n"
        "fact window = outside\nfact last = read\n",
    )
    result = run_cli("gaps", catalogue, context)
    assert result.returncode == 0
    assert result.stdout == (
        "gaps mini p\n"
        "gap beta\n"
        "blocked tools has edit, recorded read\n"
        "blocked window is inside, recorded outside\n"
    )
    assert result.stderr == ""


def test_the_gaps_command_with_no_gaps_prints_the_header_alone(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(
        tmp_path,
        "p.context",
        "context mini p\nfact tools = read, edit\n"
        "fact window = inside\nfact last = read\n",
    )
    result = run_cli("gaps", catalogue, context)
    assert result.returncode == 0
    assert result.stdout == "gaps mini p\n"


def test_refusal_parity_for_a_malformed_catalogue(tmp_path):
    catalogue = write(tmp_path, "bad.catalogue", "catalogue mini\nwander off\n")
    context = write(tmp_path, "p.context", "context mini p\nfact last = none\n")
    sheet = run_cli("sheet", catalogue, context)
    gaps = run_cli("gaps", catalogue, context)
    assert sheet.returncode == gaps.returncode == 2
    assert gaps.stdout == ""
    assert gaps.stderr == sheet.stderr
    assert "Traceback" not in gaps.stderr


def test_refusal_parity_for_a_malformed_context(tmp_path):
    catalogue = write(tmp_path, "mini.catalogue", MINI)
    context = write(tmp_path, "p.context", "context mini\n")
    sheet = run_cli("sheet", catalogue, context)
    gaps = run_cli("gaps", catalogue, context)
    assert sheet.returncode == gaps.returncode == 2
    assert gaps.stdout == ""
    assert gaps.stderr == sheet.stderr
    assert "Traceback" not in gaps.stderr


def test_refusal_parity_for_an_unreadable_file(tmp_path):
    context = write(tmp_path, "p.context", "context mini p\nfact last = none\n")
    missing = str(tmp_path / "absent.catalogue")
    sheet = run_cli("sheet", missing, context)
    gaps = run_cli("gaps", missing, context)
    assert sheet.returncode == gaps.returncode == 2
    assert gaps.stderr == sheet.stderr
    lines = [ln for ln in gaps.stderr.splitlines() if ln.strip()]
    assert len(lines) == 1


def test_the_gaps_view_is_deterministic_and_order_independent():
    context_text = (
        "context mini p\nfact tools = read\n"
        "fact window = outside\nfact last = read\n"
    )
    first = gaps_text(CATALOGUE, context_text)
    second = gaps_text(CATALOGUE, context_text)
    assert first == second
    shuffled = parse_context(
        "context mini p\nfact last = read\n"
        "fact window = outside\nfact tools = read\n",
        "s.context",
        CATALOGUE,
    )
    assert format_gaps(shuffled, build_gaps(CATALOGUE, shuffled)) == first
