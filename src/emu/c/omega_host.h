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

// Audio: submit one stereo sample pair to the host output buffer.
// left/right are signed 16-bit PCM; host may silently drop if no audio HW.
void omega_host_audio_sample(int16_t left, int16_t right);

// ROM: ponteiro e tamanho carregados do SD card pelo kernel.
// Retorna NULL/0 se nenhuma ROM foi carregada (usa built-in como fallback).
const uint8_t* omega_host_rom_ptr(void);
size_t         omega_host_rom_size(void);
// Convenience: log "prefix: 0xXXXXXXXX" without needing sprintf.
static inline void omega_log_hex(const char* prefix, uint32_t val) {
    static const char HEX[] = "0123456789ABCDEF";
    char buf[48];
    int i = 0;
    while (prefix[i] && i < 36) { buf[i] = prefix[i]; i++; }
    buf[i++] = ':'; buf[i++] = ' ';
    buf[i++] = '0'; buf[i++] = 'x';
    for (int s = 28; s >= 0; s -= 4) buf[i++] = HEX[(val >> s) & 0xF];
    buf[i] = '\0';
    omega_host_log(buf);
}

#endif /* OMEGA_HOST_H_ */
