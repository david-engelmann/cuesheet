"""Sheet generation, the evaluation of rules over recorded facts."""

from dataclasses import dataclass


@dataclass(frozen=True)
class Cue:
    name: str
    fires: str
    fires_in_turn: str
    evidence: tuple


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


def _evaluate(action, context):
    """Return the evidence lines for an action, or None if unavailable."""
    evidence = []
    for rule_test in action.tests:
        if rule_test.fact not in context.facts:
            return None
        recorded = context.facts[rule_test.fact]
        if not _test_passes(rule_test, recorded):
            return None
        evidence.append(
            f"evidence {rule_test.fact} {rule_test.test} "
            f"{rule_test.value}, recorded "
            f"{_render_recorded(recorded)}"
        )
    return tuple(evidence)


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
