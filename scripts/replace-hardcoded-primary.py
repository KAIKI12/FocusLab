#!/usr/bin/env python3
"""
Replace hardcoded primary-blue shadows/hex with theme-aware tokens.

Patterns replaced:
  rgba(79, 140, 255, ALPHA)  -> color-mix(in srgb, var(--color-primary) PCT%, transparent)
  #4F8CFF  -> var(--color-primary)
  #7AABFF  -> var(--color-primary-light)
  #3366CC  -> var(--color-primary-dark)

Skips:
  - lines in accent swatch markup (class="sw" or data-accent=)
  - prototype/assets/tokens.css (source of truth)
  - FocusLab-主题预览.html (external reference)
"""
import os, re, sys

ROOT = r"C:/Users/zhx/Desktop/FocusLab/prototype"

# rgba(79,140,255,X) with optional spaces
RGBA_RE = re.compile(r'rgba\(\s*79\s*,\s*140\s*,\s*255\s*,\s*([0-9.]+)\s*\)', re.IGNORECASE)
HEX_MAP = [
    (re.compile(r'#4F8CFF\b', re.IGNORECASE), 'var(--color-primary)'),
    (re.compile(r'#7AABFF\b', re.IGNORECASE), 'var(--color-primary-light)'),
    (re.compile(r'#3366CC\b', re.IGNORECASE), 'var(--color-primary-dark)'),
]

SKIP_LINE_MARKERS = ('class="sw"', 'data-accent=', "'#4F8CFF'", '"#4F8CFF"')

def rgba_to_colormix(m):
    alpha = float(m.group(1))
    pct = round(alpha * 100, 1)
    if pct == int(pct):
        pct = int(pct)
    return f'color-mix(in srgb, var(--color-primary) {pct}%, transparent)'

def process_file(path):
    with open(path, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    changed = 0
    out = []
    for line in lines:
        if any(m in line for m in SKIP_LINE_MARKERS):
            out.append(line)
            continue
        new_line, n1 = RGBA_RE.subn(rgba_to_colormix, line)
        n2 = 0
        for pat, rep in HEX_MAP:
            new_line, cnt = pat.subn(rep, new_line)
            n2 += cnt
        if n1 + n2 > 0:
            changed += n1 + n2
        out.append(new_line)
    if changed:
        with open(path, 'w', encoding='utf-8', newline='\n') as f:
            f.writelines(out)
    return changed

total_files = 0
total_changes = 0
for dirpath, _, filenames in os.walk(ROOT):
    for name in filenames:
        if not name.endswith('.html'):
            continue
        p = os.path.join(dirpath, name)
        n = process_file(p)
        if n:
            total_files += 1
            total_changes += n
            print(f"{n:4d}  {p}")

print()
print(f"Files modified: {total_files}")
print(f"Total replacements: {total_changes}")
