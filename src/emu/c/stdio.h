// src/emu/c/stdio.h
// Stub bare-metal — declara apenas o que o Omega2 usa.
// Implementações reais estão em omega_stubs.c.
#ifndef OMEGA_STDIO_H_
#define OMEGA_STDIO_H_

#include <stdint.h>
#include <stdarg.h>
#include <stddef.h>

int printf(const char* fmt, ...);
int sprintf(char* buf, const char* fmt, ...);
int snprintf(char* buf, size_t n, const char* fmt, ...);
int fprintf(void* stream, const char* fmt, ...);

#endif /* OMEGA_STDIO_H_ */
