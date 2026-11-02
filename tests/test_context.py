"""CU-1 and CU-4. The context format, its refusals, harness match."""

import pytest

from cuesheet.catalogue import parse_catalogue
from cuesheet.context import parse_context
from cuesheet.errors import ContextError
from tests.test_catalogue import MINI

CATALOGUE = parse_catalogue(MINI, "mini.catalogue")

FULL = """context mini start
fact tools = read, edit
fact window = inside
fact last = read
"""


def test_a_valid_context_records_each_fact():
    context = parse_context(FULL, "start.context", CATALOGUE)
    assert context.harness == "mini"
    assert context.decision_point == "start"
    assert context.facts["tools"] == frozenset({"read", "edit"})
    assert context.facts["window"] == "inside"
    assert context.facts["last"] == "read"


def test_a_context_may_record_a_subset_of_the_facts():
    context = parse_context(
        "context mini start\nfact tools = read\n", "sub.context", CATALOGUE
    )
    assert set(context.facts) == {"tools"}


def test_a_missing_header_is_refused():
    with pytest.raises(ContextError):
        parse_context("fact tools = read\n", "bad.context", CATALOGUE)


def test_a_header_with_no_decision_point_is_refused():
    with pytest.raises(ContextError):
        parse_context("context mini\nfact tools = read\n", "bad.context", CATALOGUE)


def test_a_fact_the_catalogue_never_declares_is_refused():
    text = FULL.replace("fact last = read", "fact weather = calm")
    with pytest.raises(ContextError) as caught:
        parse_context(text, "bad.context", CATALOGUE)
    assert "weather" in str(caught.value)


def test_a_fact_recorded_twice_is_refused():
    text = FULL + "fact window = outside\n"
    with pytest.raises(ContextError) as caught:
        parse_context(text, "bad.context", CATALOGUE)
    assert "window" in str(caught.value)


def test_a_recorded_value_outside_the_declared_values_is_refused():
    text = FULL.replace("fact window = inside", "fact window = sideways")
    with pytest.raises(ContextError):
        parse_context(text, "bad.context", CATALOGUE)


def test_a_recorded_token_outside_the_declared_tokens_is_refused():
    text = FULL.replace("fact tools = read, edit", "fact tools = read, fly")
    with pytest.raises(ContextError) as caught:
        parse_context(text, "bad.context", CATALOGUE)
    assert "fly" in str(caught.value)


def test_a_line_matching_no_form_is_refused():
    with pytest.raises(ContextError):
        parse_context(FULL + "wander off\n", "bad.context", CATALOGUE)


def test_a_harness_mismatch_is_refused_naming_both_harnesses():
    text = FULL.replace("context mini start", "context other start")
    with pytest.raises(ContextError) as caught:
        parse_context(text, "bad.context", CATALOGUE)
    message = str(caught.value)
    assert "other" in message and "mini" in message
