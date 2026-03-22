// src/emu/c/stdlib.h
// Stub bare-metal — declara apenas o que o Omega2 usa.
// Implementações reais estão em omega_stubs.c.
#ifndef OMEGA_STDLIB_H_
#define OMEGA_STDLIB_H_

#include <stddef.h>

void* malloc(size_t size);
void  free(void* ptr);
void* realloc(void* ptr, size_t size);
void* calloc(size_t n, size_t size);

#endif /* OMEGA_STDLIB_H_ */
