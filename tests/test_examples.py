"""CU-12. The committed examples and the sheets they produce."""

import pathlib

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.sheet import build_sheet, format_sheet

EXAMPLES = pathlib.Path(__file__).resolve().parents[1] / "examples"


def produce(context_name):
    catalogue = parse_catalogue(
        (EXAMPLES / "stagehand.catalogue").read_text(), "stagehand.catalogue"
    )
    context = parse_context(
        (EXAMPLES / context_name).read_text(), context_name, catalogue
    )
    return format_sheet(context, build_sheet(catalogue, context))


def test_the_ordinary_example_regenerates_its_committed_sheet():
    produced = produce("ordinary.context")
    committed = (EXAMPLES / "ordinary.sheet").read_text()
    assert produced == committed


def test_the_restricted_example_regenerates_its_committed_sheet():
    produced = produce("restricted.context")
    committed = (EXAMPLES / "restricted.sheet").read_text()
    assert produced == committed


def test_the_example_sheets_list_exactly_the_available_actions():
    ordinary = produce("ordinary.context")
    names = [
        line[len("cue "):]
        for line in ordinary.splitlines()
        if line.startswith("cue ")
    ]
    assert names == [
        "ask-human",
        "edit-file",
        "read-file",
        "run-command",
        "search-corpus",
        "send-update",
    ]
    restricted = produce("restricted.context")
    names = [
        line[len("cue "):]
        for line in restricted.splitlines()
        if line.startswith("cue ")
    ]
    assert names == ["read-file", "search-corpus"]
