#!/usr/bin/env python3
"""Second pass: replace functional hex colors with theme tokens."""
import os, re

ROOT = r"C:/Users/zhx/Desktop/FocusLab/prototype"

# Exact hex (case-insensitive) → token
MAP = [
    (r'#FAAD14\b', 'var(--color-gold)'),
    (r'#FF8C00\b', 'var(--color-warning)'),
    (r'#52C41A\b', 'var(--color-success)'),
    (r'#F56C6C\b', 'var(--color-q1)'),
    (r'#EAF1FF\b', 'var(--color-primary-soft)'),
    (r'#FFF4D1\b', 'var(--color-gold-soft)'),
    (r'#8C8C8C\b', 'var(--color-neutral)'),
]

# rgba variants of those same colors → color-mix
RGBA = [
    (re.compile(r'rgba\(\s*250\s*,\s*173\s*,\s*20\s*,\s*([0-9.]+)\s*\)'), '--color-gold'),   # FAAD14
    (re.compile(r'rgba\(\s*255\s*,\s*140\s*,\s*0\s*,\s*([0-9.]+)\s*\)'),  '--color-warning'),  # FF8C00
    (re.compile(r'rgba\(\s*82\s*,\s*196\s*,\s*26\s*,\s*([0-9.]+)\s*\)'),  '--color-success'),  # 52C41A
    (re.compile(r'rgba\(\s*245\s*,\s*108\s*,\s*108\s*,\s*([0-9.]+)\s*\)'),'--color-q1'),
]

# Skip any line from the swatch markup in settings.html
SKIP_LINE_MARKERS = ('class="sw"', 'data-accent=')

def to_mix(m, token):
    alpha = float(m.group(1))
    pct = round(alpha * 100, 1)
    if pct == int(pct): pct = int(pct)
    return f'color-mix(in srgb, var({token}) {pct}%, transparent)'

hex_patterns = [(re.compile(p, re.IGNORECASE), rep) for p, rep in MAP]

total = 0
for dp, _, files in os.walk(ROOT):
    for n in files:
        if not n.endswith('.html'): continue
        p = os.path.join(dp, n)
        with open(p, 'r', encoding='utf-8') as f: lines = f.readlines()
        changed = 0
        new_lines = []
        for line in lines:
            if any(m in line for m in SKIP_LINE_MARKERS):
                new_lines.append(line); continue
            new_line = line
            for pat, rep in hex_patterns:
                new_line, c = pat.subn(rep, new_line); changed += c
            for pat, tok in RGBA:
                new_line, c = pat.subn(lambda m, t=tok: to_mix(m, t), new_line); changed += c
            new_lines.append(new_line)
        if changed:
            with open(p, 'w', encoding='utf-8', newline='\n') as f: f.writelines(new_lines)
            total += changed
            print(f'{changed:4d}  {p}')

print(f'\nTotal: {total}')
