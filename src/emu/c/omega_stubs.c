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
// Nota: o hook dispara ANTES de REG_IR ser carregado (ver m68kcpu.c linha 666-672).
// REG_PC aponta para a instrução prestes a executar — lemos o opcode via memória.
// Para STOP (0x4E72 xxxx): emite EVT_CPU_STOP(PC, new_SR) onde new_SR é o imediato.
#include "omega2/cpu/m68k.h"
#include "omega2/memory/Memory.h"
#include "omega2/debug/omega_probe.h"

void cpu_instr_callback(void) {
    unsigned int pc = m68k_get_reg(NULL, M68K_REG_PC);
    if(cpu_read_word(pc) == 0x4E72) {
        probe_emit(EVT_CPU_STOP, pc, cpu_read_word(pc + 2));
    }
}
