#!/usr/bin/env python3
"""Stage built release artifacts into an empty caller-selected prefix."""
import argparse
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--prefix', type=Path, required=True)
parser.add_argument('--artifacts', type=Path, default=ROOT / 'target/release')
args = parser.parse_args()
prefix = args.prefix.resolve()
if prefix.exists() and any(prefix.iterdir()):
    raise SystemExit(f'staging prefix must be empty: {prefix}')
for name in ['libreplia_c.a', 'libreplia_c.so']:
    if not (args.artifacts / name).is_file():
        raise SystemExit(f'build release binding first: missing {name}')
(prefix / 'include').mkdir(parents=True, exist_ok=True)
(prefix / 'lib/pkgconfig').mkdir(parents=True)
shutil.copy2(ROOT / 'include/replia.h', prefix / 'include/replia.h')
for name in ['libreplia_c.a', 'libreplia_c.so']:
    shutil.copy2(args.artifacts / name, prefix / 'lib' / name)
(prefix / 'lib/pkgconfig/replia.pc').write_text('''prefix=${pcfiledir}/../..
libdir=${prefix}/lib
includedir=${prefix}/include

Name: replia
Description: REPLIA terminal interaction C binding (pre-release ABI 1)
Version: 0.1.0-dev.0
Libs: -L${libdir} -lreplia_c
Libs.private: -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc
Cflags: -I${includedir}
''')
(prefix / 'share/licenses/replia').mkdir(parents=True)
shutil.copy2(ROOT / 'LICENSE', prefix / 'share/licenses/replia/LICENSE')
print(prefix)
