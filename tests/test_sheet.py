"""CU-5 to CU-9. Evaluation, missing facts, evidence, order, empty."""

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.sheet import build_sheet, format_sheet
from tests.test_catalogue import MINI

CATALOGUE = parse_catalogue(MINI, "mini.catalogue")

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


def sheet_names(context_text, catalogue=CATALOGUE):
    context = parse_context(context_text, "c.context", catalogue)
    return [cue.name for cue in build_sheet(catalogue, context)]


def test_every_passing_action_is_listed_and_no_other():
    names = sheet_names(
        "context mini p\n"
        "fact tools = read, edit\n"
        "fact window = inside\n"
        "fact last = read\n"
    )
    assert names == ["alpha", "beta"]


def test_one_failing_test_in_a_conjunction_excludes_the_action():
    names = sheet_names(
        "context mini p\n"
        "fact tools = edit\n"
        "fact window = outside\n"
        "fact last = read\n"
    )
    assert "beta" not in names
    assert names == []


def test_a_missing_fact_makes_the_action_unavailable():
    names = sheet_names(
        "context mini p\nfact tools = edit\nfact last = read\n"
    )
    assert "beta" not in names


def test_a_missing_fact_is_not_assumed_from_the_other_tests():
    names = sheet_names(
        "context mini p\nfact window = inside\nfact last = read\n"
    )
    assert names == []


def test_lacks_tests_read_the_recorded_facts():
    catalogue = parse_catalogue(DELTA, "delta.catalogue")
    names = sheet_names(
        "context mini p\nfact tools = read\nfact last = none\n",
        catalogue=catalogue,
    )
    assert names == ["alpha", "delta"]
    names = sheet_names(
        "context mini p\nfact tools = read\nfact last = read\n",
        catalogue=catalogue,
    )
    assert "delta" in names


def test_an_is_not_test_lists_the_action_when_the_value_differs():
    catalogue = parse_catalogue(EPSILON, "epsilon.catalogue")
    context = parse_context(
        "context mini p\nfact window = outside\n",
        "c.context",
        catalogue,
    )
    cues = build_sheet(catalogue, context)
    assert [cue.name for cue in cues] == ["epsilon"]
    assert cues[0].evidence == (
        "evidence window is-not inside, recorded outside",
    )


def test_an_is_not_test_excludes_the_action_when_the_value_matches():
    catalogue = parse_catalogue(EPSILON, "epsilon.catalogue")
    names = sheet_names(
        "context mini p\nfact window = inside\n",
        catalogue=catalogue,
    )
    assert "epsilon" not in names
    assert names == []


def test_a_cue_carries_its_fires_text_and_ordered_evidence():
    context = parse_context(
        "context mini p\n"
        "fact tools = edit, read\n"
        "fact window = inside\n"
        "fact last = read\n",
        "c.context",
        CATALOGUE,
    )
    cues = {cue.name: cue for cue in build_sheet(CATALOGUE, context)}
    beta = cues["beta"]
    assert beta.fires == "beta fires text"
    assert beta.fires_in_turn == "beta turn text"
    assert beta.evidence == (
        "evidence tools has edit, recorded edit, read",
        "evidence window is inside, recorded inside",
    )


def test_sheet_order_and_input_order_independence():
    shuffled = """catalogue mini
fact last values none, read
fact window values inside, outside
fact tools set-of read, edit
action beta
requires tools has edit
requires window is inside
fires beta fires text
fires-in-turn beta turn text
action alpha
requires tools has read
fires alpha fires text
fires-in-turn alpha turn text
"""
    other = parse_catalogue(shuffled, "shuffled.catalogue")
    one = parse_context(
        "context mini p\nfact tools = read, edit\n"
        "fact window = inside\nfact last = read\n",
        "a.context",
        CATALOGUE,
    )
    two = parse_context(
        "context mini p\nfact last = read\n"
        "fact window = inside\nfact tools = edit, read\n",
        "b.context",
        other,
    )
    first = format_sheet(one, build_sheet(CATALOGUE, one))
    second = format_sheet(two, build_sheet(other, two))
    assert first == second
    assert first == format_sheet(one, build_sheet(CATALOGUE, one))
    assert [cue.name for cue in build_sheet(CATALOGUE, one)] == ["alpha", "beta"]


def test_an_empty_sheet_is_the_header_alone():
    context = parse_context(
        "context mini p\nfact last = none\n", "c.context", CATALOGUE
    )
    cues = build_sheet(CATALOGUE, context)
    assert cues == ()
    assert format_sheet(context, cues) == "sheet mini p\n"
