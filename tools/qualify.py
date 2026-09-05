#!/usr/bin/env python3
"""Complete R2 qualification, including the clean-worktree closure gate."""
import argparse
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--work', type=Path)
p.add_argument('--allow-dirty', action='store_true', help='development only; does not pass the clean closure gate')
a = p.parse_args()
work = a.work or Path(tempfile.mkdtemp(prefix='replai-r2-'))
work.mkdir(parents=True, exist_ok=True)
commands = [
    ['cargo', 'fmt', '--check'],
    ['cargo', 'check', '--workspace', '--all-targets'],
    ['cargo', 'test', '--workspace', '--all-targets'],
    ['cargo', 'clippy', '--workspace', '--all-targets', '--all-features', '--', '-D', 'warnings'],
    ['cargo', 'test', '--doc'],
    ['python3', 'tools/qualify_c.py', '--work', str(work)],
    ['cargo', 'test', '--workspace', '--release'],
    ['git', 'diff', '--check'],
]
for i, command in enumerate(commands):
    print('GATE ' + ' '.join(command), flush=True)
    result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    (work / f'gate-{i}.log').write_text(result.stdout)
    print(result.stdout, flush=True)
    result.check_returncode()
status = subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True)
if status:
    if not a.allow_dirty:
        raise SystemExit('Closure requires a clean repository:\n' + status)
    print('DEVELOPMENT RUN: clean-worktree gate remains unqualified')
else:
    print('Clean-worktree gate: PASS')
print('Evidence: ' + str(work.resolve()))
