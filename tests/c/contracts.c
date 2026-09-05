/* SPDX-License-Identifier: MIT
 * Executable ABI misuse, caller-buffer, FD ownership and repeated lifecycle proof.
 * This translation unit uses only the installed header plus Linux/standard C APIs.
 */
#define _GNU_SOURCE
#include "replia.h"
#include <assert.h>
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/ioctl.h>
#include <termios.h>
#include <unistd.h>

struct tty { int master, slave; struct termios saved; };
struct snapshot { uint8_t bytes[256]; size_t length, cursor; };
static struct termios attributes(int fd) {
    struct termios t;
    memset(&t, 0, sizeof t);
    assert(tcgetattr(fd, &t) == 0);
    return t;
}
static int same_attributes(struct termios a, struct termios b) {
    return !memcmp(&a, &b, sizeof a);
}
static struct tty terminal(void) {
    struct tty t;
    t.master = posix_openpt(O_RDWR | O_NOCTTY | O_NONBLOCK);
    assert(t.master >= 0 && grantpt(t.master) == 0 && unlockpt(t.master) == 0);
    t.slave = open(ptsname(t.master), O_RDWR | O_NOCTTY);
    assert(t.slave >= 0);
    struct winsize size = { .ws_row = 24, .ws_col = 80 };
    assert(ioctl(t.slave, TIOCSWINSZ, &size) == 0);
    struct termios initial = attributes(t.slave);
    initial.c_iflag ^= IXOFF;
    initial.c_cc[VMIN] = 3; initial.c_cc[VTIME] = 7;
    assert(tcsetattr(t.slave, TCSANOW, &initial) == 0);
    t.saved = attributes(t.slave);
    return t;
}
static void drain(int fd) {
    uint8_t bytes[4096];
    while (read(fd, bytes, sizeof bytes) > 0) {}
    assert(errno == EAGAIN || errno == EWOULDBLOCK);
}
static int fd_count(void) {
    DIR *dir = opendir("/proc/self/fd");
    assert(dir);
    int n = 0;
    struct dirent *e;
    while ((e = readdir(dir))) if (strcmp(e->d_name, ".") && strcmp(e->d_name, "..")) n++;
    assert(closedir(dir) == 0);
    return n;
}
static replia_config config(void) {
    replia_config c = {0};
    c.struct_size = sizeof c; c.abi_version = REPLIA_C_ABI_VERSION;
    c.max_input_bytes = 64; c.history_entries = 3;
    return c;
}
static replia_event event(void) {
    replia_event e = {0};
    e.struct_size = sizeof e; e.abi_version = REPLIA_C_ABI_VERSION;
    return e;
}
static struct snapshot snapshot(replia_handle *h) {
    struct snapshot s;
    memset(&s, 0, sizeof s);
    assert(replia_draft_copy(h, s.bytes, sizeof s.bytes, &s.length, &s.cursor) == REPLIA_OK);
    return s;
}
static void exact(struct snapshot a, struct snapshot b) {
    assert(a.length == b.length && a.cursor == b.cursor);
    assert(!memcmp(a.bytes, b.bytes, a.length));
}
#define EXPECT(label, expression, wanted) do { \
    struct snapshot before = snapshot(h); \
    struct termios tty_before = attributes(t.slave); \
    replia_status actual = (expression); \
    assert(actual == (wanted)); \
    exact(before, snapshot(h)); \
    assert(same_attributes(tty_before, attributes(t.slave))); \
    printf("MISUSE %s expected=%d observed=%d draft_cursor_unchanged=1 terminal_unchanged=1 reusable=1\n", label, (int)(wanted), (int)actual); \
} while (0)
static replia_event send_bytes(replia_handle *h, int master, const uint8_t *bytes, size_t n) {
    assert(write(master, bytes, n) == (ssize_t)n);
    replia_event e = event();
    for (size_t i = 0; i < n; i++) {
        e = event();
        assert(replia_poll(h, 20, &e) == REPLIA_OK);
    }
    drain(master);
    return e;
}
static void copy_contract(replia_handle *h, int submitted, const uint8_t *expected, size_t length) {
    uint8_t output[128]; size_t required = 999, cursor = 999;
#define COPY(p, cap) (submitted ? replia_submitted_copy(h, p, cap, &required) : replia_draft_copy(h, p, cap, &required, &cursor))
    assert(COPY(NULL, 0) == REPLIA_OK && required == length);
    memset(output, 0xa5, sizeof output);
    assert(COPY(output, 0) == REPLIA_BUFFER_TOO_SMALL && required == length);
    for (size_t i = 0; i < sizeof output; i++) assert(output[i] == 0xa5);
    assert(COPY(output, length - 1) == REPLIA_BUFFER_TOO_SMALL && required == length);
    for (size_t i = 0; i < sizeof output; i++) assert(output[i] == 0xa5);
    assert(COPY(output, length) == REPLIA_OK && required == length);
    assert(!memcmp(output, expected, length) && output[length] == 0xa5);
    memset(output, 0xa5, sizeof output);
    assert(COPY(output, sizeof output) == REPLIA_OK && required == length);
    assert(!memcmp(output, expected, length) && output[length] == 0xa5);
    printf("COPY %s exact_required=%zu query=OK small=BUFFER_TOO_SMALL no_partial=1 utf8_multiline_exact=1\n", submitted ? "submitted" : "draft", required);
#undef COPY
}
int main(void) {
    struct tty t = terminal(), other = terminal();
    int initial_fds = fd_count();
    replia_config c = config(), bad;
    replia_handle *h = NULL, *out = NULL;
    assert(replia_create(&c, &h) == REPLIA_OK && h);
    const uint8_t sample[] = "é界\né";
    assert(replia_set_draft(h, sample, sizeof sample - 1) == REPLIA_OK);
    replia_event e = event(), event_before;
    uint8_t buffer[256]; size_t required = 0, cursor = 0;
    uint32_t version = 0;
    assert(replia_abi_version(&version) == REPLIA_OK && version == REPLIA_C_ABI_VERSION);
    assert(replia_status_text(REPLIA_OK, buffer, sizeof buffer, &required) == REPLIA_OK);
    assert(required == 7 && !memcmp(buffer, "success", 7));
    EXPECT("unknown_status", replia_status_text(999, buffer, sizeof buffer, &required), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_handle", replia_clear(NULL), REPLIA_INVALID_ARGUMENT);
    _Alignas(uint32_t) uint8_t misaligned[8] = {0};
    EXPECT("misaligned_version_out", replia_abi_version((uint32_t *)(void *)(misaligned + 1)), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_out", replia_create(&c, NULL), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_config", replia_create(NULL, &out), REPLIA_INVALID_ARGUMENT);
    EXPECT("occupied_out", replia_create(&c, &h), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_version_out", replia_abi_version(NULL), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_destroy_slot", replia_destroy(NULL), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_destroy_handle", replia_destroy(&out), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_text_nonzero", replia_set_draft(h, NULL, 1), REPLIA_INVALID_ARGUMENT);
    EXPECT("length_overflow", replia_set_draft(h, (const uint8_t *)1, SIZE_MAX), REPLIA_INVALID_ARGUMENT);
    bad = c; bad.abi_version++;
    EXPECT("config_version", replia_create(&bad, &out), REPLIA_ABI_MISMATCH);
    bad = c; bad.struct_size = 4;
    EXPECT("config_undersized", replia_create(&bad, &out), REPLIA_ABI_MISMATCH);
    bad = c; bad.reserved[1] = 1;
    EXPECT("config_reserved", replia_create(&bad, &out), REPLIA_INVALID_ARGUMENT);
    bad = c; bad.max_input_bytes = UINT64_MAX;
    EXPECT("config_length_overflow", replia_create(&bad, &out), REPLIA_INVALID_ARGUMENT);
    EXPECT("invalid_utf8", replia_set_draft(h, (const uint8_t *)"\xff", 1), REPLIA_INVALID_UTF8);
    uint8_t large[1025]; memset(large, 'x', sizeof large);
    EXPECT("oversized_input", replia_set_draft(h, large, sizeof large), REPLIA_CAPACITY);
    EXPECT("oversized_prompt", replia_prompt(h, large, sizeof large, NULL, 0, NULL, 0), REPLIA_CAPACITY);
    EXPECT("prompt_controls", replia_prompt(h, (const uint8_t *)"\033", 1, NULL, 0, NULL, 0), REPLIA_INVALID_TEXT);
    EXPECT("poll_before_open", replia_poll(h, 0, &e), REPLIA_INVALID_STATE);
    EXPECT("complete_before_open", replia_complete(h, 0, 0, NULL, 0), REPLIA_INVALID_STATE);
    EXPECT("submitted_before_submit", replia_submitted_copy(h, buffer, sizeof buffer, &required), REPLIA_INVALID_STATE);
    EXPECT("close_before_open", replia_close(h), REPLIA_OK);
    EXPECT("double_close", replia_close(h), REPLIA_OK);
    EXPECT("null_event", replia_poll(h, 0, NULL), REPLIA_INVALID_ARGUMENT);
    e = event(); e.abi_version++;
    EXPECT("event_version", replia_poll(h, 0, &e), REPLIA_ABI_MISMATCH);
    e = event(); e.struct_size = 4;
    EXPECT("event_undersized", replia_poll(h, 0, &e), REPLIA_ABI_MISMATCH);
    e = event(); e.reserved[0] = 1;
    EXPECT("event_reserved", replia_poll(h, 0, &e), REPLIA_INVALID_ARGUMENT);
    e = event(); e.kind = 999; event_before = e;
    EXPECT("invalid_event_tag", replia_poll(h, 0, &e), REPLIA_INVALID_ARGUMENT);
    assert(!memcmp(&e, &event_before, sizeof e));
    EXPECT("null_copy_required", replia_draft_copy(h, buffer, sizeof buffer, NULL, &cursor), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_copy_cursor", replia_draft_copy(h, buffer, sizeof buffer, &required, NULL), REPLIA_INVALID_ARGUMENT);
    EXPECT("null_copy_buffer", replia_draft_copy(h, NULL, 1, &required, &cursor), REPLIA_INVALID_ARGUMENT);
    EXPECT("buffer_too_small", replia_draft_copy(h, buffer, 1, &required, &cursor), REPLIA_BUFFER_TOO_SMALL);
    copy_contract(h, 0, sample, sizeof sample - 1);
    int null_fd = open("/dev/null", O_RDWR); assert(null_fd >= 0);
    EXPECT("non_tty", replia_open(h, null_fd, null_fd), REPLIA_UNSUITABLE_TERMINAL);
    EXPECT("mismatched_tty", replia_open(h, t.slave, other.slave), REPLIA_UNSUITABLE_TERMINAL);
    EXPECT("negative_fd", replia_open(h, -1, t.slave), REPLIA_INVALID_ARGUMENT);
    assert(close(null_fd) == 0);
    EXPECT("closed_fd", replia_open(h, null_fd, t.slave), REPLIA_INVALID_ARGUMENT);
    assert(replia_set_draft(h, NULL, 0) == REPLIA_OK);
    assert(snapshot(h).length == 0);
    printf("MISUSE null_text_zero expected=0 observed=0 draft_cleared=1 terminal_unchanged=1 reusable=1\n");
    assert(replia_set_draft(h, sample, sizeof sample - 1) == REPLIA_OK);
    int readonly = open(ptsname(t.master), O_RDONLY | O_NOCTTY); assert(readonly >= 0);
    EXPECT("partial_open_write_failure", replia_open(h, t.slave, readonly), REPLIA_IO);
    assert(close(readonly) == 0);
    assert(same_attributes(t.saved, attributes(t.slave)));
    assert(replia_open(h, t.slave, t.slave) == REPLIA_OK);
    drain(t.master);
    EXPECT("open_twice", replia_open(h, t.slave, t.slave), REPLIA_INVALID_STATE);
    assert(replia_create(&c, &out) == REPLIA_OK);
    EXPECT("second_terminal", replia_open(out, other.slave, other.slave), REPLIA_BUSY);
    assert(replia_destroy(&out) == REPLIA_OK && !out);
    EXPECT("invalid_style", replia_external_output(h, 999, NULL, 0), REPLIA_INVALID_ARGUMENT);
    EXPECT("output_controls", replia_external_output(h, REPLIA_ROLE_DEFAULT, (const uint8_t *)"\033[2J", 4), REPLIA_INVALID_TEXT);
    EXPECT("output_utf8", replia_external_output(h, REPLIA_ROLE_DEFAULT, (const uint8_t *)"\xff", 1), REPLIA_INVALID_UTF8);
    EXPECT("invalid_range", replia_complete(h, 99, 100, NULL, 0), REPLIA_INVALID_RANGE);
    EXPECT("reversed_range", replia_complete(h, 5, 1, NULL, 0), REPLIA_INVALID_RANGE);
    EXPECT("split_utf8", replia_complete(h, 1, 2, NULL, 0), REPLIA_INVALID_RANGE);
    EXPECT("split_grapheme", replia_complete(h, 7, 7, NULL, 0), REPLIA_INVALID_RANGE);
    EXPECT("completion_capacity", replia_complete(h, 0, 0, large, sizeof large), REPLIA_CAPACITY);
    EXPECT("completion_utf8", replia_complete(h, 0, 0, (const uint8_t *)"\xff", 1), REPLIA_INVALID_UTF8);
    EXPECT("set_active_draft", replia_set_draft(h, NULL, 0), REPLIA_INVALID_STATE);
    EXPECT("active_history", replia_history_add(h, NULL, 0), REPLIA_INVALID_STATE);
    assert(read(t.master, buffer, sizeof buffer) == -1 && errno == EAGAIN);
    e = send_bytes(h, t.master, (const uint8_t *)"\r", 1);
    assert(e.kind == REPLIA_EVENT_SUBMITTED && e.status == REPLIA_OK);
    assert(same_attributes(t.saved, attributes(t.slave)));
    copy_contract(h, 1, sample, sizeof sample - 1);
    int writeonly = open(ptsname(t.master), O_WRONLY | O_NOCTTY); assert(writeonly >= 0);
    assert(replia_open(h, writeonly, t.slave) == REPLIA_OK);
    assert(write(t.master, "x", 1) == 1);
    e = event(); assert(replia_poll(h, 20, &e) == REPLIA_IO);
    assert(same_attributes(t.saved, attributes(t.slave)));
    assert(close(writeonly) == 0);
    /* Host owns recovery of the deliberately unread fault-test byte. */
    assert(tcflush(t.slave, TCIFLUSH) == 0);
    printf("IO read_failure status=7 termios_restored=1 handle_reusable=1\n");
    assert(replia_open(h, t.slave, t.slave) == REPLIA_OK);
    drain(t.master);
    e = event(); assert(replia_poll(h, 0, &e) == REPLIA_OK && e.kind == REPLIA_EVENT_NONE);
    struct snapshot retained = snapshot(h);
    e = send_bytes(h, t.master, (const uint8_t *)"\xff", 1);
    assert(e.kind == REPLIA_EVENT_EDIT_REJECTED && e.status == REPLIA_INVALID_UTF8);
    exact(retained, snapshot(h));
    e = event(); e.abi_version++;
    EXPECT("interrupt_abi_mismatch", replia_interrupt(h, &e), REPLIA_ABI_MISMATCH);
    e = event(); assert(replia_interrupt(h, &e) == REPLIA_OK);
    assert(e.kind == REPLIA_EVENT_INTERRUPTED && e.status == REPLIA_OK);
    exact(retained, snapshot(h));
    assert(same_attributes(t.saved, attributes(t.slave)));
    printf("EVENTS no_event=0 rejected=5 rejection_status=2 explicit_interrupt=2 draft_cursor_unchanged=1 termios_restored=1\n");
    assert(replia_destroy(&h) == REPLIA_OK && !h);
    assert(fd_count() == initial_fds);
    /* Repeat across owners AND multiple interactions per owner, under Memcheck. */
    for (int i = 0; i < 128; i++) {
        assert(replia_create(&c, &h) == REPLIA_OK);
        for (int j = 0; j < 3; j++) {
            if (j == 1) {
                int input = dup(t.slave), output = dup(t.slave);
                assert(input >= 0 && output >= 0);
                assert(replia_open(h, input, output) == REPLIA_OK);
                assert(close(input) == 0 && close(output) == 0);
            } else assert(replia_open(h, t.slave, t.slave) == REPLIA_OK);
            drain(t.master);
            const uint8_t paste[] = "\033[200~é\r\n界\033[201~\t";
            e = send_bytes(h, t.master, paste, sizeof paste - 1);
            assert(e.kind == REPLIA_EVENT_COMPLETION_REQUESTED);
            assert(replia_complete(h, 0, 2, (const uint8_t *)"a", 1) == REPLIA_OK);
            struct snapshot before = snapshot(h);
            assert(replia_external_output(h, REPLIA_ROLE_SUCCESS, (const uint8_t *)"notice", 6) == REPLIA_OK);
            exact(before, snapshot(h));
            e = send_bytes(h, t.master, (const uint8_t *)"\r", 1);
            assert(e.kind == REPLIA_EVENT_SUBMITTED && e.text_bytes == 5 && e.cursor_bytes == 1);
            assert(replia_submitted_copy(h, buffer, sizeof buffer, &required) == REPLIA_OK);
            assert(required == 5 && !memcmp(buffer, "a\n界", 5));
            assert(replia_history_add(h, buffer, required) == REPLIA_OK);
            assert(replia_close(h) == REPLIA_OK && replia_close(h) == REPLIA_OK);
            assert(same_attributes(t.saved, attributes(t.slave)));
            assert(fcntl(t.master, F_GETFD) >= 0 && fcntl(t.slave, F_GETFD) >= 0);
            assert(fd_count() == initial_fds);
            assert(replia_clear(h) == REPLIA_OK);
        }
        assert(replia_destroy(&h) == REPLIA_OK && !h);
        assert(fd_count() == initial_fds);
    }
    printf("LIFECYCLE owners=128 open_close=384 fd_before=%d fd_after=%d caller_fds_valid=1 termios_equal=384 submitted_bytes=610ae7958c cursor_bytes=1\n", initial_fds, fd_count());
    assert(close(t.master) == 0 && close(t.slave) == 0);
    assert(close(other.master) == 0 && close(other.slave) == 0);
    puts("CONTRACTS PASS");
    return 0;
}
