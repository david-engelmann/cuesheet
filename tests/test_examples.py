"""CU-12. The committed examples and the sheets they produce."""

import pathlib

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.sheet import build_gaps, build_sheet, format_gaps, format_sheet

EXAMPLES = pathlib.Path(__file__).resolve().parents[1] / "examples"


def produce(context_name):
    catalogue = parse_catalogue(
        (EXAMPLES / "stagehand.catalogue").read_text(), "stagehand.catalogue"
    )
    context = parse_context(
        (EXAMPLES / context_name).read_text(), context_name, catalogue
    )
    return format_sheet(context, build_sheet(catalogue, context))


def produce_gaps(context_name):
    catalogue = parse_catalogue(
        (EXAMPLES / "stagehand.catalogue").read_text(), "stagehand.catalogue"
    )
    context = parse_context(
        (EXAMPLES / context_name).read_text(), context_name, catalogue
    )
    return format_gaps(context, build_gaps(catalogue, context))


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


def test_the_ordinary_example_regenerates_its_committed_gaps_view():
    produced = produce_gaps("ordinary.context")
    committed = (EXAMPLES / "ordinary.gaps").read_text()
    assert produced == committed


def test_the_restricted_example_regenerates_its_committed_gaps_view():
    produced = produce_gaps("restricted.context")
    committed = (EXAMPLES / "restricted.gaps").read_text()
    assert produced == committed


def test_the_example_gaps_name_exactly_the_actions_the_sheets_leave_off():
    ordinary = produce_gaps("ordinary.context")
    names = [
        line[len("gap "):]
        for line in ordinary.splitlines()
        if line.startswith("gap ")
    ]
    assert names == ["spawn-helper"]
    restricted = produce_gaps("restricted.context")
    names = [
        line[len("gap "):]
        for line in restricted.splitlines()
        if line.startswith("gap ")
    ]
    assert names == [
        "ask-human",
        "edit-file",
        "run-command",
        "send-update",
        "spawn-helper",
    ]
    assert "blocked human-present is yes, not recorded\n" in restricted
