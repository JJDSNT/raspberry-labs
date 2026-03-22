// src/emu/c/omega_host.h
// HAL interface — implemented on the Rust side via extern "C"
#ifndef OMEGA_HOST_H_
#define OMEGA_HOST_H_

#include <stdint.h>
#include <stddef.h>

// Framebuffer provided by the kernel (ARGB8888, 800x600 or native res)
uint32_t* omega_host_framebuffer(void);
int32_t   omega_host_pitch(void);       // bytes per row

// Yield until next frame slot (calls kernel scheduler yield)
void omega_host_vsync(void);

// Log a message via kernel UART
void omega_host_log(const char* msg);

// Input: enfileira um evento de tecla vindo do USB HID (chamado pelo C).
// scancode: USB HID keycode, pressed: 1=down, 0=up
void omega_host_push_key(uint8_t scancode, int pressed);

// Input: retira um evento da fila. Retorna 1 se havia evento, 0 se vazia.
int omega_host_poll_key(uint8_t* scancode, int* pressed);

// ROM: ponteiro e tamanho carregados do SD card pelo kernel.
// Retorna NULL/0 se nenhuma ROM foi carregada (usa built-in como fallback).
const uint8_t* omega_host_rom_ptr(void);
size_t         omega_host_rom_size(void);

#endif /* OMEGA_HOST_H_ */
