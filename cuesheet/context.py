"""A recorded harness context at one decision point."""

from dataclasses import dataclass

from .catalogue import SET, VALUE
from .errors import ContextError
from .lines import meaningful_lines


@dataclass(frozen=True)
class Context:
    harness: str
    decision_point: str
    facts: dict


def parse_context(text, source, catalogue):
    """Parse one context against a catalogue or raise ContextError."""
    lines = meaningful_lines(text)
    if not lines:
        raise ContextError(f"{source} is malformed, it names no harness")
    head = lines[0].split()
    if len(head) != 3 or head[0] != "context":
        raise ContextError(
            f"{source} is malformed, the first line does not name the "
            "harness and the decision point"
        )
    harness, decision_point = head[1], head[2]
    if harness != catalogue.harness:
        raise ContextError(
            f"{source} names the harness {harness} and the catalogue "
            f"names {catalogue.harness}, and a sheet is defined only "
            "over one harness family"
        )
    recorded = {}
    for line in lines[1:]:
        parts = line.split()
        if len(parts) < 4 or parts[0] != "fact" or parts[2] != "=":
            raise ContextError(
                f"{source} is malformed, the line {line} matches no "
                "context line form"
            )
        name = parts[1]
        fact = catalogue.facts.get(name)
        if fact is None:
            raise ContextError(
                f"{source} is malformed, the fact {name} is one the "
                "catalogue never declares"
            )
        if name in recorded:
            raise ContextError(
                f"{source} is malformed, the fact {name} is recorded "
                "twice"
            )
        raw = line.split("=", 1)[1].strip()
        if fact.kind == VALUE:
            if raw not in fact.options:
                raise ContextError(
                    f"{source} is malformed, the fact {name} is "
                    f"recorded as {raw}, which the catalogue never "
                    "declares for it"
                )
            recorded[name] = raw
        else:
            tokens = tuple(part.strip() for part in raw.split(","))
            if any(not token for token in tokens):
                raise ContextError(
                    f"{source} is malformed, the fact {name} carries "
                    "an empty token"
                )
            for token in tokens:
                if token not in fact.options:
                    raise ContextError(
                        f"{source} is malformed, the fact {name} "
                        f"carries the token {token}, which the "
                        "catalogue never declares for it"
                    )
            recorded[name] = frozenset(tokens)
    return Context(harness, decision_point, recorded)


def load_context(text, source, catalogue):
    return parse_context(text, source, catalogue)
