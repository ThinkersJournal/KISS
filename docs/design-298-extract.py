# SPDX-License-Identifier: MIT OR Apache-2.0
import json
import re
import sys

root = sys.argv[1]
ops = json.load(open(root + '/conformance/corpus/op_manifest.json'))['all_ops']
spec = open(root + '/spec/ops.md', encoding='utf-8').read()

# role vocabulary (proposed, closed, append-only)
R_X = 'operand'

T = {}


def put(names, count, roles, src):
    for n in names.split():
        if n in T:
            raise SystemExit('duplicate row for op: ' + n)
        T[n] = (count, roles, src)


U = ('1', 'operand')
put('abs neg bit_not clz ctz popcount floor ceil trunc round_even frac recip rsqrt sqr sign step relu sigmoid silu mish softplus gelu gelu_tanh '
    'exp exp2 expm1 log log2 log10 log1p sqrt cbrt sin cos tan sinh cosh tanh asin acos atan asinh acosh atanh erf erfc lgamma logical_not',
    '1', 'operand', 'catalog §2.7; §6.13 body free variable `x`')
put('add sub mul div bit_and bit_or bit_xor shl shr pow hypot rem_floor rem_trunc logical_and logical_or atan2 copysign nextafter '
    'max_prop min_prop fmax_ieee fmin_ieee cmp_eq cmp_ne cmp_lt cmp_le cmp_gt cmp_ge',
    '2', 'lhs, rhs', '§6.4/§6.6/§6.9/§6.10; §6.13 body free variables `a`,`b`')
put('ldexp', '2', 'operand, exponent', '§6.13 `ldexp(a, b)`; `b` is the integer exponent')
put('select', '3', 'cond, on_true, on_false', 'KISS-OPS-6.5-0001: order `(cond, a, b)`')
put('reduce reduce_mean reduce_norm2 reduce_var reduce_std logsumexp all any argmax argmin cumsum cumprod cummax prefix_scan '
    'softmax log_softmax avg_pool max_pool im2col sort_network',
    '1', 'operand', '§6.11/§6.13 decompositions read one input; attributes carry axes/monoid/window')
put('matmul', '2', 'lhs, rhs', '§6.13 operand-ordering convention: `input(0)=lhs`, `input(1)=rhs`')
put('rms_norm', '2', 'operand, scale', '§6.13 convention: `input(0)=x`, `input(1)=gamma`; `eps`=`param(0)` is a param, not an operand')
put('layer_norm', '3', 'operand, scale, shift', '§6.13 convention: `input(0..2)=x,gamma,beta`; `eps`=`param(0)`')
put('gather', '2..3', 'data, index, base?', 'KISS-OPS-6.11-0012/-0015: index operand + OPTIONAL `base`')
put('index_select embedding', '2', 'data, index', '§6.13 `gather` with a 1-D index operand')
put('scatter scatter_add', '3', 'dest, index, updates', 'KISS-OPS-6.11-0016/-0017: `dest` and `updates` operands + index operand')
put('cadd csub cmul cdiv cpow', '2', 'lhs, rhs', '§6.18 table: complex binary')
put('cneg cabs carg cconj cexp clog csqrt cre cim', '1', 'operand', '§6.18 table: complex unary / component read')
put('cmake', '2', 'real, imag', '§6.18: lane 0 = `input(0)`, lane 1 = `input(1)`')
put('element_map', 'n (variadic)', 'operand*', 'KISS-OPS-6.11-0001: the base access over a per-element scalar body; the body names its `input(i)`')

missing = sorted(set(ops) - set(T))
extra = sorted(set(T) - set(ops))
if missing or extra:
    raise SystemExit('row set differs from all_ops: missing=%s extra=%s' % (missing, extra))

# positive control: arities we know independently from the spec's own formulas
known = {'add': '2', 'select': '3', 'neg': '1', 'layer_norm': '3', 'matmul': '2', 'scatter': '3', 'cmake': '2', 'sqr': '1'}
for k, v in known.items():
    if T[k][0] != v:
        raise SystemExit('positive control failed: %s has count %s, spec says %s' % (k, T[k][0], v))

# cross-check against the clean §6.13 bodies: free variables a,b => 2; only x => 1
s = spec[spec.index('### 6.13 Reference'):spec.index('### 6.14')]
rows = re.findall(r'^\| `([a-z0-9_]+)` \| [a-z_ ]+ \| [^|]* \| (`[^|]*`)', s, re.M)
bad = []
checked = 0
for op, body in rows:
    body = body.strip('`')
    if 'input(' in body:
        continue
    ids = set(re.findall(r'\b(a|b|x)\b', body))
    if ids == {'x'}:
        want = '1'
    elif ids == {'a', 'b'}:
        want = '2'
    else:
        continue
    checked += 1
    if T[op][0] != want:
        bad.append((op, want, T[op][0]))
print('cross-checked', checked, 'ops against §6.13 bodies; mismatches:', bad)
if checked < 40 or bad:
    raise SystemExit('cross-check failed: checked=%d mismatches=%s' % (checked, bad))


lines = spec.split('\n')
CLAUSE = {
    'select': 'KISS-OPS-6.5-0001', 'matmul': '§6.13 operand-ordering convention (ops.md:%d)',
    'rms_norm': '§6.13 operand-ordering convention', 'layer_norm': '§6.13 operand-ordering convention',
    'gather': 'KISS-OPS-6.11-0012, -0015', 'index_select': '§6.13 row + KISS-OPS-6.11-0012',
    'embedding': '§6.13 row + KISS-OPS-6.11-0012', 'scatter': 'KISS-OPS-6.11-0016, -0017',
    'scatter_add': '§6.13 row + KISS-OPS-6.11-0016, -0017', 'cmake': '§6.18 table row (lane 0 = input(0), lane 1 = input(1))',
    'element_map': 'KISS-OPS-6.11-0001',
}
for _c in 'cadd csub cmul cdiv cpow cneg cabs carg cconj cexp clog csqrt cre cim'.split():
    CLAUSE[_c] = '§6.18 table row (a,b there are the real/imag COMPONENTS of z, not operands; 6.18 intro: "For a complex operand z")'


def rows_for(op):
    out = []
    for i, l in enumerate(lines):
        m = re.match(r'\| `%s` \| (.*)$' % re.escape(op), l)
        if m and not l.startswith('| `%s` | `' % op) or (m and False):
            pass
        if re.match(r'\| `%s` \|' % re.escape(op), l):
            sec = ''
            for j in range(i, -1, -1):
                if lines[j].startswith('### '):
                    sec = lines[j][4:].split(' ')[0]
                    break
            out.append((i + 1, sec, l))
    return out


def bodytext(l):
    cells = [c.strip() for c in l.strip().strip('|').split('|')]
    return ' '.join(cells[1:])


def letters(text, first_only):
    spans = re.findall(r'`([^`]*)`', text)
    code = (spans[0] if spans else '') if first_only else ' '.join(spans)
    return set(re.findall(r'(?<![A-Za-z_0-9])(a|b|x|cond)(?![A-Za-z_0-9])', code))


want_letters = {'1': {'x'}, '2': {'a', 'b'}}
PREF = ['6.13', '6.18', '2.7']
kinds = {}
md = ['| Op | Count | Roles (position order) | Anchor: first spec row (ops.md line, §) and what it says | Evidence |', '|---|---|---|---|---|']
for o in ops:
    c, r, b = T[o]
    rs = rows_for(o)
    pick = None
    for p in PREF:
        for x in rs:
            if x[1] == p:
                pick = x
                break
        if pick:
            break
    if not pick and rs:
        pick = rs[0]
    kind = 'meaning-only'
    if o in CLAUSE:
        kind = 'clause'
    elif pick:
        allsp, first = set(), set()
        for x in rs:
            allsp |= letters(bodytext(x[2]), False)
            first |= letters(bodytext(x[2]), True)
        if c in want_letters and want_letters[c] <= allsp and first <= want_letters[c]:
            kind = 'formula'
    kinds[o] = kind
    if pick:
        txt = bodytext(pick[2]).replace('|', r'\|')
        txt = (txt[:95] + '…') if len(txt) > 95 else txt
        anchor = 'ops.md:%d (§%s) — %s' % (pick[0], pick[1], txt)
    else:
        anchor = 'none found'
    ev = kind if kind != 'clause' else 'clause: ' + CLAUSE[o]
    md.append('| `%s` | %s | %s | %s | %s |' % (o, c, r, anchor, ev))
from collections import Counter
print(Counter(kinds.values()))
open(sys.argv[2], 'w', encoding='utf-8').write('\n'.join(md) + '\n')
