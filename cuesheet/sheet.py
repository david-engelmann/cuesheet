"""The sheet and the gaps view, from one evaluation of the rules."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Cue:
    name: str
    fires: str
    fires_in_turn: str
    evidence: tuple


@dataclass(frozen=True)
class Gap:
    name: str
    blocked: tuple


def _render_recorded(recorded):
    if isinstance(recorded, frozenset):
        return ", ".join(sorted(recorded))
    return recorded


def _test_passes(rule_test, recorded):
    if rule_test.test == "is":
        return recorded == rule_test.value
    if rule_test.test == "is-not":
        return recorded != rule_test.value
    if rule_test.test == "has":
        return rule_test.value in recorded
    if rule_test.test == "lacks":
        return rule_test.value not in recorded
    raise ValueError(f"unknown rule test kind {rule_test.test}")


def _evaluate_tests(action, context):
    """Evaluate every rule test of an action against the context.

    Returns one triple per test in catalogue order, holding the test,
    the recorded value it was checked against, and whether it passed.
    The recorded value is None when the context does not record the
    fact, and a test on an unrecorded fact never passes under CU-6.
    Both views are computed from these triples, so they cannot
    disagree about an action.
    """
    results = []
    for rule_test in action.tests:
        if rule_test.fact not in context.facts:
            results.append((rule_test, None, False))
            continue
        recorded = context.facts[rule_test.fact]
        results.append(
            (rule_test, recorded, _test_passes(rule_test, recorded))
        )
    return tuple(results)


def _evaluate(action, context):
    """Return the evidence lines for an action, or None if unavailable."""
    results = _evaluate_tests(action, context)
    if not all(passed for _, _, passed in results):
        return None
    return tuple(
        f"evidence {rule_test.fact} {rule_test.test} "
        f"{rule_test.value}, recorded "
        f"{_render_recorded(recorded)}"
        for rule_test, recorded, _ in results
    )


def build_sheet(catalogue, context):
    """Return the cues available in this context, sorted by name."""
    cues = []
    for name in sorted(catalogue.actions):
        action = catalogue.actions[name]
        evidence = _evaluate(action, context)
        if evidence is not None:
            cues.append(
                Cue(action.name, action.fires, action.fires_in_turn, evidence)
            )
    return tuple(cues)


def format_sheet(context, cues):
    """Render one sheet as deterministic text."""
    lines = [f"sheet {context.harness} {context.decision_point}"]
    for cue in cues:
        lines.append(f"cue {cue.name}")
        lines.append(f"fires {cue.fires}")
        lines.append(f"fires-in-turn {cue.fires_in_turn}")
        lines.extend(cue.evidence)
    return "\n".join(lines) + "\n"


def build_gaps(catalogue, context):
    """Return the gaps, the unavailable actions and why, by name."""
    gaps = []
    for name in sorted(catalogue.actions):
        action = catalogue.actions[name]
        results = _evaluate_tests(action, context)
        if all(passed for _, _, passed in results):
            continue
        blocked = []
        for rule_test, recorded, passed in results:
            if passed:
                continue
            if recorded is None:
                blocked.append(
                    f"blocked {rule_test.fact} {rule_test.test} "
                    f"{rule_test.value}, not recorded"
                )
            else:
                blocked.append(
                    f"blocked {rule_test.fact} {rule_test.test} "
                    f"{rule_test.value}, recorded "
                    f"{_render_recorded(recorded)}"
                )
        gaps.append(Gap(action.name, tuple(blocked)))
    return tuple(gaps)


def format_gaps(context, gaps):
    """Render the gaps view as deterministic text."""
    lines = [f"gaps {context.harness} {context.decision_point}"]
    for gap in gaps:
        lines.append(f"gap {gap.name}")
        lines.extend(gap.blocked)
    return "\n".join(lines) + "\n"
