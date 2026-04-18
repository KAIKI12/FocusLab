#!/usr/bin/env python3
"""Replace dark text variant hex with new *-text tokens."""
import os, re

ROOT = r"C:/Users/zhx/Desktop/FocusLab/prototype"

MAP = [
    (r'#AD6800\b', 'var(--color-gold-text)'),    # 深金 → gold-text
    (r'#C2410C\b', 'var(--color-warning-text)'), # 深橙 → warning-text (q3也用这个)
    (r'#389E0D\b', 'var(--color-success-text)'), # 深绿 → success-text
]
SKIP = ('class="sw"', 'data-accent=')

patterns = [(re.compile(p, re.IGNORECASE), rep) for p, rep in MAP]
total = 0
for dp, _, files in os.walk(ROOT):
    for n in files:
        if not n.endswith('.html'): continue
        p = os.path.join(dp, n)
        with open(p, 'r', encoding='utf-8') as f: lines = f.readlines()
        changed = 0
        new_lines = []
        for line in lines:
            if any(m in line for m in SKIP):
                new_lines.append(line); continue
            nl = line
            for pat, rep in patterns:
                nl, c = pat.subn(rep, nl); changed += c
            new_lines.append(nl)
        if changed:
            with open(p, 'w', encoding='utf-8', newline='\n') as f: f.writelines(new_lines)
            total += changed
            print(f'{changed:4d}  {p}')
print(f'\nTotal: {total}')
