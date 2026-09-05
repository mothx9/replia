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
for name in ['libreplai_c.a', 'libreplai_c.so']:
    if not (args.artifacts / name).is_file():
        raise SystemExit(f'build release binding first: missing {name}')
(prefix / 'include').mkdir(parents=True, exist_ok=True)
(prefix / 'lib/pkgconfig').mkdir(parents=True)
shutil.copy2(ROOT / 'include/replai.h', prefix / 'include/replai.h')
for name in ['libreplai_c.a', 'libreplai_c.so']:
    shutil.copy2(args.artifacts / name, prefix / 'lib' / name)
(prefix / 'lib/pkgconfig/replai.pc').write_text('''prefix=${pcfiledir}/../..
libdir=${prefix}/lib
includedir=${prefix}/include

Name: replai
Description: REPLAI terminal interaction C binding (pre-release ABI 1)
Version: 0.1.0-dev.0
Libs: -L${libdir} -lreplai_c
Libs.private: -lgcc_s -lutil -lrt -lpthread -lm -ldl -lc
Cflags: -I${includedir}
''')
(prefix / 'share/licenses/replai').mkdir(parents=True)
shutil.copy2(ROOT / 'LICENSE', prefix / 'share/licenses/replai/LICENSE')
print(prefix)
