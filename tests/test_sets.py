"""CU-19. The set reading of the format lists and of the rules."""

import pytest

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.errors import CatalogueError, ContextError
from cuesheet.sheet import build_gaps, build_sheet, format_gaps, format_sheet

BASE = """catalogue sets
fact tools set-of read, edit
fact window values inside, outside
action alpha
requires tools has read
fires alpha fires text
fires-in-turn alpha turn text
action beta
requires tools has edit
requires window is inside
fires beta fires text
fires-in-turn beta turn text
"""

CONTEXT = """context sets start
fact tools = read
fact window = inside
"""

FULL_CONTEXT = """context sets start
fact tools = read, edit
fact window = inside
"""


def views(catalogue_text, context_text):
    catalogue = parse_catalogue(catalogue_text, "sets.catalogue")
    context = parse_context(context_text, "sets.context", catalogue)
    sheet = format_sheet(context, build_sheet(catalogue, context))
    gaps = format_gaps(context, build_gaps(catalogue, context))
    return sheet, gaps


def test_a_repeated_option_in_a_values_list_reads_as_written_once():
    repeated = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside, inside, outside",
    )
    assert views(repeated, CONTEXT) == views(BASE, CONTEXT)


def test_a_repeated_token_in_a_set_of_list_reads_as_written_once():
    repeated = BASE.replace(
        "fact tools set-of read, edit", "fact tools set-of read, edit, edit"
    )
    assert views(repeated, CONTEXT) == views(BASE, CONTEXT)


def test_a_repeated_token_in_a_recorded_set_reads_as_written_once():
    repeated = FULL_CONTEXT.replace(
        "fact tools = read, edit", "fact tools = read, edit, read"
    )
    assert views(BASE, repeated) == views(BASE, FULL_CONTEXT)


def test_whitespace_around_a_comma_is_no_part_of_an_option_or_token():
    snug = BASE.replace(
        "fact tools set-of read, edit", "fact tools set-of read,edit"
    )
    loose = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside , outside",
    )
    snug_context = FULL_CONTEXT.replace(
        "fact tools = read, edit", "fact tools = read ,edit"
    )
    assert views(snug, CONTEXT) == views(BASE, CONTEXT)
    assert views(loose, CONTEXT) == views(BASE, CONTEXT)
    assert views(BASE, snug_context) == views(BASE, FULL_CONTEXT)


def test_an_empty_option_in_a_fact_declaration_is_refused():
    text = BASE.replace(
        "fact window values inside, outside",
        "fact window values inside,, outside",
    )
    with pytest.raises(CatalogueError) as caught:
        parse_catalogue(text, "bad.catalogue")
    assert "window" in str(caught.value)


def test_an_empty_token_in_a_set_of_list_is_refused():
    text = BASE.replace(
        "fact tools set-of read, edit", "fact tools set-of read, edit,"
    )
    with pytest.raises(CatalogueError) as caught:
        parse_catalogue(text, "bad.catalogue")
    assert "tools" in str(caught.value)


def test_an_empty_token_in_a_recorded_set_is_refused():
    catalogue = parse_catalogue(BASE, "sets.catalogue")
    text = FULL_CONTEXT.replace(
        "fact tools = read, edit", "fact tools = read, edit,"
    )
    with pytest.raises(ContextError) as caught:
        parse_context(text, "bad.context", catalogue)
    assert "tools" in str(caught.value)


def test_a_repeated_passing_test_prints_its_evidence_line_once():
    repeated = BASE.replace(
        "action alpha\nrequires tools has read\n",
        "action alpha\nrequires tools has read\nrequires tools has read\n",
    )
    sheet, gaps = views(repeated, CONTEXT)
    assert (sheet, gaps) == views(BASE, CONTEXT)
    assert sheet.count("evidence tools has read, recorded read") == 1


def test_a_repeated_failing_test_prints_its_blocked_line_once():
    repeated = BASE.replace(
        "action beta\nrequires tools has edit\n",
        "action beta\nrequires tools has edit\nrequires tools has edit\n",
    )
    sheet, gaps = views(repeated, CONTEXT)
    assert (sheet, gaps) == views(BASE, CONTEXT)
    assert gaps.count("blocked tools has edit, recorded read") == 1


def test_the_parser_records_a_repeated_test_once_in_first_order():
    text = BASE.replace(
        "action alpha\nrequires tools has read\n",
        "action alpha\nrequires window is inside\n"
        "requires tools has read\nrequires window is inside\n",
    )
    catalogue = parse_catalogue(text, "sets.catalogue")
    stored = catalogue.actions["alpha"].tests
    assert [(t.fact, t.test, t.value) for t in stored] == [
        ("window", "is", "inside"),
        ("tools", "has", "read"),
    ]
    sheet, _ = views(text, CONTEXT)
    evidence = [line for line in sheet.splitlines() if "evidence" in line]
    assert evidence == [
        "evidence window is inside, recorded inside",
        "evidence tools has read, recorded read",
    ]


def test_two_actions_stating_the_same_test_each_keep_their_own():
    text = BASE.replace(
        "action beta\nrequires tools has edit\n",
        "action beta\nrequires tools has read\n",
    )
    catalogue = parse_catalogue(text, "sets.catalogue")
    alpha = catalogue.actions["alpha"].tests
    beta = catalogue.actions["beta"].tests
    assert [(t.fact, t.test, t.value) for t in alpha] == [
        ("tools", "has", "read")
    ]
    assert [(t.fact, t.test, t.value) for t in beta] == [
        ("tools", "has", "read"),
        ("window", "is", "inside"),
    ]
    sheet, _ = views(text, CONTEXT)
    assert sheet.count("evidence tools has read, recorded read") == 2
