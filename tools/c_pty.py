#!/usr/bin/env python3
"""Drive an external C PROCESS through Linux PTYs; use an independent VT oracle."""
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import termios
import time

class Console:
    def __init__(self, command, oracle, *, once=True, flags=(), plain=False, dumb=False, size=(24, 80), preexec=None, environment=None):
        self.master, self.slave = pty.openpty()
        self.size = size
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack('HHHH', *size, 0, 0))
        initial = termios.tcgetattr(self.slave)
        initial[0] ^= termios.IXOFF
        initial[6][termios.VMIN] = 3
        initial[6][termios.VTIME] = 7
        termios.tcsetattr(self.slave, termios.TCSANOW, initial)
        self.before = termios.tcgetattr(self.slave)
        env = (environment or os.environ).copy()
        env.pop('NO_COLOR', None)
        env.pop('LD_LIBRARY_PATH', None)
        env['TERM'] = 'dumb' if dumb else 'xterm-256color'
        if plain:
            env['NO_COLOR'] = ''
        self.p = subprocess.Popen([*command, '--trace', *(['--once'] if once else []), *flags],
            stdin=self.slave, stdout=self.slave, stderr=subprocess.PIPE, env=env, preexec_fn=preexec)
        self.oracle = oracle
        self.operations = [f'R {size[0]} {size[1]}']
        self.bytes = bytearray()
        self.errors = bytearray()
        self.pending = bytearray()
        self.events = []
        self.actions = []
        self.open_event = self.wait('OPEN')
        assert self.before != termios.tcgetattr(self.slave), 'open did not change terminal attributes'
    def pump(self, timeout=0.02):
        ready, _, _ = select.select([self.master, self.p.stderr.fileno()], [], [], timeout)
        for fd in ready:
            data = os.read(fd, 65536)
            if fd == self.master:
                if data:
                    self.bytes.extend(data)
                    self.operations.append('D ' + data.hex())
            else:
                self.errors.extend(data)
                self.pending.extend(data)
                while b'\n' in self.pending:
                    line, _, remainder = self.pending.partition(b'\n')
                    self.pending[:] = remainder
                    fields = line.decode().split(' ', 4)
                    if fields[0] in ('OPEN', 'EVENT', 'COMPLETE', 'OUTPUT', 'PALETTE'):
                        assert len(fields) == 5, line
                        self.events.append({'label': fields[0], 'kind': int(fields[1]), 'status': int(fields[2]), 'cursor_bytes': int(fields[3]), 'text': bytes.fromhex(fields[4]).decode()})
    def wait(self, label, *, after=0, kind=None):
        deadline = time.monotonic() + 20
        while time.monotonic() < deadline:
            for e in self.events[after:]:
                if e['label'] == label and (kind is None or e['kind'] == kind):
                    self.pump(0)
                    return e
            if self.p.poll() is not None:
                self.pump(0)
                raise AssertionError((label, self.p.returncode, self.errors.decode(), self.bytes))
            self.pump()
        raise AssertionError(('PTY timeout', label, self.errors.decode(), self.bytes))
    def send(self, data):
        if isinstance(data, str): data = data.encode()
        self.actions.append({'input_hex': data.hex()})
        mark = len(self.events)
        assert os.write(self.master, data) == len(data)
        return mark
    def complete_barrier(self, data=b''):
        mark = self.send(data + b'\t')
        return self.wait('COMPLETE', after=mark)
    def resize(self, rows, cols):
        self.pump(0)
        self.actions.append({'resize': [rows, cols]})
        self.operations.append(f'R {rows} {cols}')
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack('HHHH', rows, cols, 0, 0))
        self.size = rows, cols
    def screen(self):
        self.pump(0.02)
        output = subprocess.run([str(self.oracle)], input='\n'.join(self.operations)+'\n', text=True, capture_output=True, check=True).stdout
        state = {'cells': {}}
        for line in output.splitlines():
            fields = line.split(' ')
            if fields[0] == 'cursor': state['cursor'] = [int(fields[1]), int(fields[2])]
            elif fields[0] == 'text': state['text'] = bytes.fromhex(fields[1]).decode()
            elif fields[0] == 'cell': state['cells'][f'{fields[1]},{fields[2]}'] = fields[3:]
            else: state[fields[0]] = fields[1]
        assert state['alternate'] == 'false'
        assert state['background_default'] == 'true'
        return state
    def finish(self, data, kind):
        mark = self.send(data)
        outcome = self.wait('EVENT', after=mark, kind=kind)
        deadline = time.monotonic() + 20
        while self.p.poll() is None and time.monotonic() < deadline: self.pump()
        assert self.p.wait(timeout=1) == 0, self.errors.decode()
        self.pump(0)
        assert b'DESTROY 0\n' in self.errors
        assert self.before == termios.tcgetattr(self.slave), 'termios before != after clean destroy'
        state = self.screen()
        assert state['paste'] == 'false'
        return outcome, state
    def dispose(self):
        if self.p.poll() is None:
            self.p.kill(); self.p.wait()
        self.p.stderr.close()
        os.close(self.master); os.close(self.slave)

def run_suite(command, oracle, records, output, preexec=None, environment=None):
    records = dict(line.split('\t') for line in Path(records).read_text().splitlines())
    records = {k: bytes.fromhex(v) for k, v in records.items()}
    observations = []
    def run(name, body, **kwargs):
        c = Console(command, oracle, preexec=preexec, environment=environment, **kwargs)
        try:
            observed = body(c)
            observations.append({'scenario': name, 'actions': c.actions, 'events': c.events,
                'observation': observed, 'termios_before_equals_after': c.before == termios.tcgetattr(c.slave),
                'exit': c.p.returncode, 'output_hex': c.bytes.hex()})
            assert c.p.returncode == 0
            print(f'PTY {name}: PASS; text={observed["text"]!r}; cursor={observed["cursor"]}; termios before == after; exit 0', flush=True)
        finally: c.dispose()
    def prompt(c, key):
        state = c.screen()
        assert bytes(c.bytes) == b'\x1b[?2004h' + records[key]
        assert state['text'] == 'demo> ' and state['cursor'] == [0, 6]
        assert state['cells']['0,0'][1] == ('Idx(81)' if key == 'styled_prompt' else 'Default')
        c.finish(b'\x04', 3)
        return state
    run('C01 styled prompt', lambda c: prompt(c, 'styled_prompt'))
    run('C02 NO_COLOR', lambda c: prompt(c, 'no_color'), plain=True)
    run('TERM=dumb', lambda c: prompt(c, 'dumb'), dumb=True)
    def utf8(c):
        e = c.complete_barrier('hé界🌍\x1b[D\x7fX'.encode())
        assert (e['text'], e['cursor_bytes']) == ('héX🌍', 4)
        state = c.screen()
        assert state['text'] == 'demo> héX🌍' and state['cursor'] == [0, 9]
        submitted, _ = c.finish(b'\r', 1)
        assert submitted['text'] == 'héX🌍'
        assert 'echo: héX🌍'.encode() in c.bytes
        return state
    run('C03 UTF-8 cursor edit', utf8)
    def history(c):
        mark = c.send(b'earlier\r')
        c.wait('EVENT', after=mark, kind=1)
        c.wait('OPEN', after=mark)
        e = c.complete_barrier(b'draft\x1b[D\x1b[A\x1b[B')
        assert (e['text'], e['cursor_bytes']) == ('draft', 4)
        state = c.screen()
        assert state['text'].endswith('demo> draft') and state['cursor'] == [3, 10]
        mark = c.send(b'\x03'); c.wait('EVENT', after=mark, kind=2); c.wait('OPEN', after=mark)
        c.finish(b'\x04', 3)
        return state
    run('C04 history draft return / C14 reopen', history, once=False)
    def completion(c):
        e = c.complete_barrier(b'wor')
        assert (e['text'], e['cursor_bytes']) == ('world', 5)
        assert any(x['kind'] == 4 and x['text'] == 'wor' for x in c.events)
        state = c.screen()
        assert state['text'] == 'demo> world' and state['cursor'] == [0, 11]
        e, _ = c.finish(b'\r', 1); assert e['text'] == 'world'
        assert b'echo: world' in c.bytes
        return state
    run('C05 C-selected completion', completion)
    def paste(c):
        e = c.complete_barrier('\x1b[200~é\r\n界\x1b[201~'.encode())
        assert (e['text'], e['cursor_bytes']) == ('é\n界', 6)
        assert not any(x['kind'] == 1 for x in c.events)
        state = c.screen()
        assert state['text'] == 'demo> é\n... 界' and state['cursor'] == [1, 6]
        e, _ = c.finish(b'\r', 1); assert e['text'] == 'é\n界'
        assert 'echo: é\r\n界'.encode() in c.bytes
        return state
    run('C06 bracketed multiline paste', paste)
    def resize(c):
        c.complete_barrier('\x1b[200~ab界\r\nline 🌍\x1b[201~\x1b[D'.encode())
        c.resize(12, 9)
        e = c.complete_barrier()
        assert (e['text'], e['cursor_bytes']) == ('ab界\nline 🌍', 11)
        state = c.screen()
        assert state['text'] == 'demo> ab\n界\n... line \n🌍', state
        assert state['cursor'] == [3, 0], state
        e, _ = c.finish(b'X\r', 1); assert e['text'] == 'ab界\nline X🌍'
        return state
    run('C07 resize active draft', resize, size=(12, 12))
    def external(c):
        e = c.complete_barrier('ab界\x1b[D'.encode())
        assert (e['text'], e['cursor_bytes']) == ('ab界', 2)
        c.wait('OUTPUT')
        state = c.screen()
        assert state['text'] == 'notice: the draft is still yours\ndemo> ab界'
        assert state['cursor'] == [1, 8]
        assert state['cells']['0,0'][1] == 'Idx(245)'
        e, _ = c.finish(b'X\r', 1); assert e['text'] == 'abX界'
        return state
    run('C08 external output mid-draft', external, flags=['--notice'])
    def interrupt(c):
        e, state = c.finish(b'hello\x03', 2)
        assert e['text'] == 'hello'
        assert state['text'] == 'demo> hello^C' and state['cursor'] == [1, 0]
        return state
    run('C09 Ctrl-C', interrupt)
    def eof(c):
        e, state = c.finish(b'\x04', 3)
        assert e['text'] == '' and state['cursor'] == [1, 0]
        return state
    run('C10 empty Ctrl-D', eof)
    def delete(c):
        e = c.complete_barrier(b'abc\x04\x01\x04')
        assert (e['text'], e['cursor_bytes']) == ('bc', 0)
        state = c.screen()
        assert state['text'] == 'demo> bc' and state['cursor'] == [0, 6]
        e, _ = c.finish(b'\r', 1); assert e['text'] == 'bc'
        return state
    run('C11 nonempty Ctrl-D', delete)
    def submit(c):
        e, state = c.finish(b'hello\r', 1)
        assert e['text'] == 'hello' and e['kind'] == 1
        assert state['text'] == 'demo> hello\necho: hello' and state['cursor'] == [3, 0]
        return state
    run('C12 submit / C13 restoration / C15 clean destroy', submit)
    def clear(c):
        c.complete_barrier(b'draft\x1b[D\x0c')
        state = c.screen()
        assert state['text'] == 'demo> draft' and state['cursor'] == [0, 10], state
        c.finish(b'\r', 1)
        return state
    run('Ctrl-L redraw', clear)
    def palette(c):
        c.wait('PALETTE')
        state = c.screen()
        for row, (color, bold) in enumerate([('Default','false'),('Idx(250)','true'),('Idx(81)','false'),('Idx(245)','false'),('Idx(114)','false'),('Idx(179)','false'),('Idx(203)','false')]):
            cell = state['cells'][f'{row},0']
            assert (cell[1], cell[3]) == (color, bold), state
        c.finish(b'\x04', 3)
        return state
    run('all generic palette roles', palette, flags=['--palette'])
    Path(output).write_text(json.dumps(observations, indent=2, ensure_ascii=False)+'\n')
    return observations

if __name__ == '__main__':
    import argparse
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--demo', required=True)
    p.add_argument('--oracle', required=True)
    p.add_argument('--records', required=True)
    p.add_argument('--evidence', required=True)
    a = p.parse_args()
    run_suite([a.demo], a.oracle, a.records, a.evidence)
