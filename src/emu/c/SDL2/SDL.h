// src/emu/c/SDL2/SDL.h
// Stub bare-metal — substitui SDL2 para o Omega2 compilar sem SDL.
#ifndef OMEGA_SDL_STUB_H_
#define OMEGA_SDL_STUB_H_

#include <stdint.h>

// SDL_GetPerformanceCounter — usado em EventQueue.c::StartExecution()
// StartExecution() não é chamado no path bare-metal, mas precisa compilar.
static inline uint64_t SDL_GetPerformanceCounter(void) { return 0; }
static inline uint64_t SDL_GetPerformanceFrequency(void) { return 1; }

#endif /* OMEGA_SDL_STUB_H_ */
