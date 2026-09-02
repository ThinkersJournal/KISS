"""Extract a workflow step's `run:` block so it can be EXECUTED, not read (#338, #369).

Shell inside YAML is the least-tested code in this repository: nothing imports it, no linter
reads it, and its only execution is on a runner where a mistake presents as a confusing red
rather than as a failure. Both the ratchet step and the MSRV leg are tested by extracting
their block and running it under `bash -e` with the external commands stubbed.

ONE COPY, because two would drift (#372 review). `test_kiss_ratchet_step.py` and
`test_kiss_msrv_step.py` each carried an identical extractor. Nothing would have failed if
one were fixed and the other not -- and the failure is silent: a mis-extracted block still
parses and still runs, so its controls keep passing against a script the runner never sees.

THE DEDENT OFFSET IS MEASURED, not assumed to be two. A hardcoded step mis-dedents under any
other valid YAML indentation, and leading whitespace is insignificant to bash, so the damage
is invisible.
"""


def run_block(text, step):
    """The named step's `run: |` script, dedented. Raises AssertionError if it is missing.

    Raising rather than returning "" is deliberate: a renamed step would otherwise turn
    every control built on it into a vacuous pass.
    """
    lines = text.splitlines()
    i = next((n for n, l in enumerate(lines) if l.strip() == "- name: " + step), None)
    if i is None:
        raise AssertionError("step %r not found" % (step,))
    j = next((n for n in range(i + 1, len(lines)) if lines[n].strip().startswith("run:")), None)
    if j is None or not lines[j].strip().endswith("|"):
        raise AssertionError("step %r has no block `run: |`" % (step,))
    indent = len(lines[j]) - len(lines[j].lstrip())
    body = []
    for l in lines[j + 1:]:
        if l.strip() and (len(l) - len(l.lstrip())) <= indent:
            break
        body.append(l)
    first = next((l for l in body if l.strip()), None)
    if first is None:
        raise AssertionError("step %r has an EMPTY `run:` block" % (step,))
    off = len(first) - len(first.lstrip())
    return "\n".join(l[off:] if len(l) > off else "" for l in body)
