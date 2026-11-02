"""The action catalogue, the fixed offer of one harness family."""

from dataclasses import dataclass

from .errors import CatalogueError
from .lines import meaningful_lines

VALUE = "values"
SET = "set-of"
TESTS_FOR_KIND = {VALUE: ("is", "is-not"), SET: ("has", "lacks")}


@dataclass(frozen=True)
class Fact:
    name: str
    kind: str
    options: tuple


@dataclass(frozen=True)
class RuleTest:
    fact: str
    test: str
    value: str


@dataclass(frozen=True)
class Action:
    name: str
    tests: tuple
    fires: str
    fires_in_turn: str


@dataclass(frozen=True)
class Catalogue:
    harness: str
    facts: dict
    actions: dict





def _value_list(raw, source, what):
    values = tuple(part.strip() for part in raw.split(","))
    if not values or any(not value for value in values):
        raise CatalogueError(
            f"{source} is malformed, the {what} carries an empty value"
        )
    return values


def _check_test(fact, test, value, source):
    if fact is None:
        raise CatalogueError(
            f"{source} is malformed, a rule test names a fact the "
            "catalogue never declares"
        )
    if test not in TESTS_FOR_KIND[fact.kind]:
        raise CatalogueError(
            f"{source} is malformed, the test {test} does not read a "
            f"fact of kind {fact.kind}, in the rule for {fact.name}"
        )
    if value not in fact.options:
        raise CatalogueError(
            f"{source} is malformed, the test on {fact.name} names "
            f"{value}, which the fact never declares"
        )


def _finish_action(pending, source):
    name, tests, fires, fires_in_turn = pending
    if not tests:
        raise CatalogueError(
            f"{source} is malformed, the action {name} carries no "
            "rule test"
        )
    if fires is None:
        raise CatalogueError(
            f"{source} is malformed, the action {name} carries no "
            "fires line"
        )
    if fires_in_turn is None:
        raise CatalogueError(
            f"{source} is malformed, the action {name} carries no "
            "fires-in-turn line"
        )
    return Action(name, tuple(tests), fires, fires_in_turn)


def parse_catalogue(text, source):
    """Parse one catalogue or raise CatalogueError naming the defect."""
    lines = meaningful_lines(text)
    if not lines:
        raise CatalogueError(f"{source} is malformed, it names no harness")
    head = lines[0].split()
    if len(head) != 2 or head[0] != "catalogue":
        raise CatalogueError(
            f"{source} is malformed, the first line does not name the "
            "harness, it must read catalogue and a harness name"
        )
    harness = head[1]
    facts = {}
    actions = {}
    pending = None
    for line in lines[1:]:
        parts = line.split()
        if parts[0] == "fact" and len(parts) >= 4 and parts[2] in (VALUE, SET):
            if pending is not None:
                raise CatalogueError(
                    f"{source} is malformed, the fact {parts[1]} is "
                    "declared after the first action"
                )
            name, kind = parts[1], parts[2]
            if name in facts:
                raise CatalogueError(
                    f"{source} is malformed, the fact {name} is "
                    "declared twice"
                )
            options = _value_list(
                " ".join(parts[3:]), source, f"fact {name}"
            )
            facts[name] = Fact(name, kind, options)
        elif parts[0] == "action" and len(parts) == 2:
            if pending is not None:
                done = _finish_action(pending, source)
                actions[done.name] = done
            name = parts[1]
            if name in actions or (pending and pending[0] == name):
                raise CatalogueError(
                    f"{source} is malformed, the action {name} is "
                    "declared twice"
                )
            pending = [name, [], None, None]
        elif parts[0] == "requires" and len(parts) == 4 and pending is not None:
            _, fact_name, test, value = parts
            _check_test(facts.get(fact_name), test, value, source)
            pending[1].append(RuleTest(fact_name, test, value))
        elif parts[0] == "fires" and pending is not None and len(line) > 6:
            if pending[2] is not None:
                raise CatalogueError(
                    f"{source} is malformed, the action {pending[0]} "
                    "carries two fires lines"
                )
            pending[2] = line[len("fires "):]
        elif (
            parts[0] == "fires-in-turn"
            and pending is not None
            and len(line) > 14
        ):
            if pending[3] is not None:
                raise CatalogueError(
                    f"{source} is malformed, the action {pending[0]} "
                    "carries two fires-in-turn lines"
                )
            pending[3] = line[len("fires-in-turn "):]
        else:
            raise CatalogueError(
                f"{source} is malformed, the line {line} matches no "
                "catalogue line form"
            )
    if pending is not None:
        done = _finish_action(pending, source)
        actions[done.name] = done
    return Catalogue(harness, facts, actions)


def load_catalogue(text, source):
    return parse_catalogue(text, source)
