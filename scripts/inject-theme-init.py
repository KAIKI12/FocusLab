#!/usr/bin/env python3
"""
Inject global theme init script into every prototype HTML file's <head>.
Idempotent: if the marker is already present, skip.
"""
import os, re

ROOT = r"C:/Users/zhx/Desktop/FocusLab/prototype"
MARKER = "/* fl-theme-init v1.2.4 */"

SNIPPET = '''<script>/* fl-theme-init v1.2.4 · global theme + accent persistence */
(function(){function apply(){try{var t=localStorage.getItem('fl-theme');var a=localStorage.getItem('fl-accent');var d=document.documentElement;if(t==='auto'){var pd=window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches;d.dataset.theme=pd?'dark':'light';}else if(t){d.dataset.theme=t;}if(a&&a!=='default'){d.dataset.accentTheme=a;}else{delete d.dataset.accentTheme;}}catch(e){}}apply();try{window.addEventListener('storage',function(e){if(e.key==='fl-theme'||e.key==='fl-accent')apply();});}catch(e){}})();
</script>
'''

# Pattern: insert right after the first </title>
TITLE_RE = re.compile(r'(</title>\s*\n)', re.IGNORECASE)

def process(path):
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    if MARKER in content:
        return 'skip'
    m = TITLE_RE.search(content)
    if not m:
        return 'no-title'
    new_content = content[:m.end()] + SNIPPET + content[m.end():]
    with open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(new_content)
    return 'ok'

count = {'ok': 0, 'skip': 0, 'no-title': 0}
for dirpath, dirnames, filenames in os.walk(ROOT):
    for name in filenames:
        if name.endswith('.html'):
            p = os.path.join(dirpath, name)
            r = process(p)
            count[r] += 1
            print(f"{r:8s} {p}")
print()
print(f"Summary: {count}")
