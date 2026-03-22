// src/emu/c/omega_stubs.c
// Bare-metal stubs for C stdlib/IO functions used by Omega2.
#include <stdint.h>
#include <stddef.h>
#include "omega_host.h"

// printf → drop output (omega_host_log handles logging)
int printf(const char* fmt, ...) {
    (void)fmt;
    return 0;
}

int sprintf(char* buf, const char* fmt, ...) {
    (void)buf; (void)fmt;
    if (buf) buf[0] = '\0';
    return 0;
}

// malloc/free — Omega2 should not call these after our Memory.c changes,
// but provide stubs to satisfy the linker.
void* malloc(size_t size) { (void)size; return NULL; }
void  free(void* ptr)     { (void)ptr; }
void* realloc(void* ptr, size_t size) { (void)ptr; (void)size; return NULL; }
void* calloc(size_t n, size_t size)   { (void)n; (void)size; return NULL; }

// cpu_instr_callback — chamado pelo Musashi a cada instrução (M68K_INSTRUCTION_HOOK).
// Stub vazio: sem disassembler em bare-metal.
void cpu_instr_callback(void) {}
