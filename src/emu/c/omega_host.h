// src/emu/c/omega_host.h
// HAL interface — implemented on the Rust side via extern "C"
#ifndef OMEGA_HOST_H_
#define OMEGA_HOST_H_

#include <stdint.h>

// Framebuffer provided by the kernel (ARGB8888, 800x600 or native res)
uint32_t* omega_host_framebuffer(void);
int32_t   omega_host_pitch(void);       // bytes per row

// Yield until next frame slot (calls kernel scheduler yield)
void omega_host_vsync(void);

// Log a message via kernel UART
void omega_host_log(const char* msg);

// Input: poll one key event. Returns 1 if event available, 0 if empty.
// scancode: Amiga raw keycode, pressed: 1=down, 0=up
int omega_host_poll_key(uint8_t* scancode, int* pressed);

#endif /* OMEGA_HOST_H_ */
