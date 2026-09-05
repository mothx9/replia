/* SPDX-License-Identifier: MIT
 * A neutral C host loop. Build using only an installed replia.h and library.
 * --notice emits output during editing; --once exits after one terminal outcome.
 * --trace writes machine-readable diagnostics to stderr for PTY qualification.
 */
#define _POSIX_C_SOURCE 200809L
#include "replia.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

static int tracing;
static int check(replia_status status) {
    if (status == REPLIA_OK) return 1;
    uint8_t text[128];
    size_t n = 0;
    if (replia_status_text(status, text, sizeof text, &n) == REPLIA_OK)
        fprintf(stderr, "error %d: %.*s\n", (int)status, (int)n, (const char *)text);
    else fprintf(stderr, "error %d\n", (int)status);
    return 0;
}
static uint8_t *draft(replia_handle *h, size_t *n, size_t *cursor) {
    if (!check(replia_draft_copy(h, NULL, 0, n, cursor))) return NULL;
    uint8_t *text = malloc(*n + 1);
    if (!text) return NULL;
    if (!check(replia_draft_copy(h, text, *n, n, cursor))) { free(text); return NULL; }
    text[*n] = 0; /* Only this C-owned convenience buffer adds a terminator. */
    return text;
}
static int trace(replia_handle *h, const char *label, uint32_t kind, int32_t status) {
    if (!tracing) return 1;
    size_t n, cursor;
    uint8_t *text = draft(h, &n, &cursor);
    if (!text) return 0;
    fprintf(stderr, "%s %u %d %zu ", label, (unsigned)kind, (int)status, cursor);
    for (size_t i = 0; i < n; i++) fprintf(stderr, "%02x", (unsigned)text[i]);
    fputc('\n', stderr);
    fflush(stderr);
    free(text);
    return 1;
}
static double now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t) != 0) return 0;
    return (double)t.tv_sec + (double)t.tv_nsec / 1000000000.0;
}
int main(int argc, char **argv) {
    int once = 0, notice = 0, notice_sent = 0, palette = 0, exit_status = 1;
    replia_handle *h = NULL;
    for (int i = 1; i < argc; i++) {
        if (!strcmp(argv[i], "--once")) once = 1;
        else if (!strcmp(argv[i], "--notice")) notice = 1;
        else if (!strcmp(argv[i], "--trace")) tracing = 1;
        else if (!strcmp(argv[i], "--palette")) palette = 1;
        else { fprintf(stderr, "unknown option: %s\n", argv[i]); return 2; }
    }
    uint32_t version = 0;
    if (!check(replia_abi_version(&version)) || version != REPLIA_C_ABI_VERSION) return 1;
    replia_config config = {0};
    config.struct_size = sizeof config;
    config.abi_version = REPLIA_C_ABI_VERSION;
    config.max_input_bytes = 65536;
    config.history_entries = 100;
    if (!check(replia_create(&config, &h))) goto done;
    if (!check(replia_prompt(h, (const uint8_t *)"demo", 4, NULL, 0,
                            (const uint8_t *)"... ", 4))) goto done;
    for (;;) {
        if (fflush(stdout) != 0 || !check(replia_open(h, STDIN_FILENO, STDOUT_FILENO))) goto done;
        if (!trace(h, "OPEN", 0, 0)) goto done;
        if (palette) {
            const char *labels[] = {"default", "strong", "accent", "dim", "success", "warning", "error"};
            for (uint32_t role = 0; role < 7; role++)
                if (!check(replia_external_output(h, role, (const uint8_t *)labels[role], strlen(labels[role])))) goto done;
            if (!trace(h, "PALETTE", 0, 0)) goto done;
            palette = 0;
        }
        double started = now();
        replia_event event;
        for (;;) {
            if (notice && !notice_sent && now() - started >= 1.0) {
                const uint8_t message[] = "notice: the draft is still yours";
                if (!check(replia_external_output(h, REPLIA_ROLE_DIM, message, sizeof message - 1))) goto done;
                if (!trace(h, "OUTPUT", 0, 0)) goto done;
                notice_sent = 1;
            }
            memset(&event, 0, sizeof event);
            event.struct_size = sizeof event;
            event.abi_version = REPLIA_C_ABI_VERSION;
            if (!check(replia_poll(h, 100, &event))) goto done;
            if (event.kind == REPLIA_EVENT_NONE) continue;
            if (!trace(h, "EVENT", event.kind, event.status)) goto done;
            if (event.kind == REPLIA_EVENT_COMPLETION_REQUESTED) {
                size_t n, cursor, matches = 0;
                const char *selected = NULL;
                const char *words[] = {"hello", "help", "world"};
                uint8_t *text = draft(h, &n, &cursor);
                if (!text) goto done;
                for (size_t i = 0; i < sizeof words / sizeof words[0]; i++) {
                    if (n <= strlen(words[i]) && !memcmp(words[i], text, n)) {
                        matches++; selected = words[i];
                    }
                }
                free(text);
                if (matches == 1 && !check(replia_complete(h, 0, n, (const uint8_t *)selected, strlen(selected)))) goto done;
                if (!trace(h, "COMPLETE", event.kind, 0)) goto done;
            } else if (event.kind == REPLIA_EVENT_EDIT_REJECTED) {
                const uint8_t message[] = "input rejected";
                if (!check(replia_external_output(h, REPLIA_ROLE_WARNING, message, sizeof message - 1))) goto done;
            } else break;
        }
        /* These outcomes already restored and released terminal ownership. */
        if (!check(replia_close(h))) goto done;
        if (event.kind == REPLIA_EVENT_SUBMITTED) {
            size_t n = 0;
            if (!check(replia_submitted_copy(h, NULL, 0, &n))) goto done;
            uint8_t *text = malloc(n ? n : 1);
            if (!text) goto done;
            int copied = check(replia_submitted_copy(h, text, n, &n));
            int admitted = copied && (!n || check(replia_history_add(h, text, n)));
            int printed = 0;
            if (admitted) printed = fputs("echo: ", stdout) >= 0 && fwrite(text, 1, n, stdout) == n && fputs("\n\n", stdout) >= 0;
            free(text);
            if (!admitted || !printed) goto done;
        }
        if (once || event.kind == REPLIA_EVENT_END_OF_INPUT) break;
        if (!check(replia_clear(h))) goto done;
    }
    exit_status = 0;
done:
    if (h && !check(replia_destroy(&h))) exit_status = 1;
    if (tracing) fprintf(stderr, "DESTROY %d\n", exit_status);
    return exit_status;
}
