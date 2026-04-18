#!/usr/bin/env python3
"""Replace SVG stop-color primary hex with CSS vars."""
import os, re

ROOT = r"C:/Users/zhx/Desktop/FocusLab/prototype"

PATS = [
    (re.compile(r'stop-color="#4F8CFF"', re.IGNORECASE), 'stop-color="var(--color-primary)"'),
    (re.compile(r'stop-color="#7AABFF"', re.IGNORECASE), 'stop-color="var(--color-primary-light)"'),
    (re.compile(r'stop-color="#3366CC"', re.IGNORECASE), 'stop-color="var(--color-primary-dark)"'),
]

total = 0
for dirpath, _, files in os.walk(ROOT):
    for n in files:
        if not n.endswith('.html'): continue
        p = os.path.join(dirpath, n)
        with open(p, 'r', encoding='utf-8') as f: c = f.read()
        orig = c
        for pat, rep in PATS:
            c = pat.sub(rep, c)
        if c != orig:
            with open(p, 'w', encoding='utf-8', newline='\n') as f: f.write(c)
            diff = sum(1 for _ in re.finditer(r'stop-color="var\(--color-primary', c)) - sum(1 for _ in re.finditer(r'stop-color="var\(--color-primary', orig))
            total += diff
            print(f"  updated  {p}")
print(f"Total: {total}")
