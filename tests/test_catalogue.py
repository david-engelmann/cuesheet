"""CU-2 and CU-3. The catalogue format and its load time refusals."""

import pytest

from cuesheet.catalogue import parse_catalogue
from cuesheet.errors import CatalogueError

MINI = """catalogue mini
fact tools set-of read, edit
fact window values inside, outside
fact last values none, read
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


def test_a_valid_catalogue_declares_facts_and_actions():
    catalogue = parse_catalogue(MINI, "mini.catalogue")
    assert catalogue.harness == "mini"
    assert catalogue.facts["tools"].kind == "set-of"
    assert catalogue.facts["tools"].options == ("read", "edit")
    assert catalogue.facts["window"].kind == "values"
    beta = catalogue.actions["beta"]
    assert [(t.fact, t.test, t.value) for t in beta.tests] == [
        ("tools", "has", "edit"),
        ("window", "is", "inside"),
    ]
    assert beta.fires == "beta fires text"
    assert beta.fires_in_turn == "beta turn text"


def test_a_missing_header_is_refused():
    with pytest.raises(CatalogueError):
        parse_catalogue("fact tools set-of read\n", "bad.catalogue")


def test_a_fact_declared_twice_is_refused():
    text = MINI.replace(
        "fact window values inside, outside\n",
        "fact window values inside, outside\nfact window values inside, outside\n",
    )
    with pytest.raises(CatalogueError) as caught:
        parse_catalogue(text, "bad.catalogue")
    assert "window" in str(caught.value)


def test_an_action_declared_twice_is_refused():
    text = MINI + (
        "action alpha\nrequires tools has read\n"
        "fires again\nfires-in-turn again\n"
    )
    with pytest.raises(CatalogueError) as caught:
        parse_catalogue(text, "bad.catalogue")
    assert "alpha" in str(caught.value)


def test_a_rule_naming_an_undeclared_fact_is_refused():
    text = MINI.replace("requires window is inside", "requires weather is inside")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")


def test_a_test_whose_form_mismatches_the_fact_kind_is_refused():
    with pytest.raises(CatalogueError):
        parse_catalogue(
            MINI.replace("requires tools has edit", "requires tools is edit"),
            "bad.catalogue",
        )
    with pytest.raises(CatalogueError):
        parse_catalogue(
            MINI.replace("requires window is inside", "requires window has inside"),
            "bad.catalogue",
        )


def test_a_tested_value_outside_the_declared_options_is_refused():
    text = MINI.replace("requires window is inside", "requires window is sideways")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")


def test_an_action_with_no_rule_test_is_refused():
    text = MINI.replace("requires tools has read\n", "")
    with pytest.raises(CatalogueError) as caught:
        parse_catalogue(text, "bad.catalogue")
    assert "alpha" in str(caught.value)


def test_an_action_missing_its_fires_line_is_refused():
    text = MINI.replace("fires alpha fires text\n", "")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")


def test_an_action_missing_its_fires_in_turn_line_is_refused():
    text = MINI.replace("fires-in-turn beta turn text\n", "")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")


def test_a_line_matching_no_form_is_refused():
    with pytest.raises(CatalogueError):
        parse_catalogue(MINI + "wander off\n", "bad.catalogue")


def test_a_requires_line_before_any_action_is_refused():
    text = MINI.replace("action alpha", "requires tools has read\naction alpha")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")


def test_fires_text_after_a_tab_separator_is_kept_whole():
    text = MINI.replace("fires alpha fires text", "fires\talpha fires text")
    catalogue = parse_catalogue(text, "tab.catalogue")
    assert catalogue.actions["alpha"].fires == "alpha fires text"


def test_a_fires_line_with_no_text_is_refused():
    text = MINI.replace("fires alpha fires text\n", "fires\n")
    with pytest.raises(CatalogueError):
        parse_catalogue(text, "bad.catalogue")
