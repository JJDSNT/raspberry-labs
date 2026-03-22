// src/emu/c/omega_input.c
//
// Bridge USB HID → CIA do Amiga.
//
// TinyUSB chama os callbacks tuh_hid_*_cb quando um dispositivo HID é
// conectado/desconectado ou quando um relatório é recebido.
// Para cada evento de tecla detectado, chamamos omega_host_push_key()
// que empurra para a fila Rust lida por omega_host_poll_key() em omega_glue.c.
//
// Protocolo boot-class keyboard (8 bytes):
//   byte 0 : modifier mask (LCtrl=0x01 LShift=0x02 LAlt=0x04 LGui=0x08
//                           RCtrl=0x10 RShift=0x20 RAlt=0x40 RGui=0x80)
//   byte 1 : reservado
//   bytes 2-7: até 6 keycodes simultâneos (0x00 = nenhum)
//
// USB HID keycodes coincidem com os valores que pressKey()/releaseKey()
// esperam — nenhuma tradução adicional necessária.

#include <stdint.h>
#include <stdbool.h>
#include "tusb.h"
#include "omega_host.h"

// Modifier bits → HID keycodes correspondentes
static const uint8_t k_mod_hid[8] = {
    0xE0, // LCtrl
    0xE1, // LShift
    0xE2, // LAlt
    0xE3, // LGui  (Left Amiga)
    0xE4, // RCtrl
    0xE5, // RShift
    0xE6, // RAlt
    0xE7, // RGui  (Right Amiga)
};

// Último relatório recebido (para detecção de delta)
static uint8_t s_prev[8] = {0};

static bool key_in_report(const uint8_t* report, uint8_t kc) {
    for (int i = 2; i < 8; i++) {
        if (report[i] == kc) return true;
    }
    return false;
}

// ---------------------------------------------------------------------------
// Callbacks TinyUSB HID host
// ---------------------------------------------------------------------------

void tuh_hid_mount_cb(uint8_t dev_addr, uint8_t instance,
                       uint8_t const* desc_report, uint16_t desc_len) {
    (void)desc_report;
    (void)desc_len;

    // Força protocolo boot para obter relatórios de 8 bytes fixos.
    // Funciona com a maioria dos teclados USB padrão.
    if (tuh_hid_interface_protocol(dev_addr, instance) == HID_ITF_PROTOCOL_KEYBOARD) {
        tuh_hid_set_protocol(dev_addr, instance, HID_PROTOCOL_BOOT);
    }

    tuh_hid_receive_report(dev_addr, instance);
}

void tuh_hid_umount_cb(uint8_t dev_addr, uint8_t instance) {
    (void)dev_addr;
    (void)instance;

    // Solta todas as teclas modificadoras ao desconectar
    for (int b = 0; b < 8; b++) {
        if (s_prev[0] & (1 << b)) {
            omega_host_push_key(k_mod_hid[b], 0);
        }
    }
    // Solta keycodes normais
    for (int i = 2; i < 8; i++) {
        if (s_prev[i]) {
            omega_host_push_key(s_prev[i], 0);
        }
    }
    for (int i = 0; i < 8; i++) s_prev[i] = 0;
}

void tuh_hid_report_received_cb(uint8_t dev_addr, uint8_t instance,
                                  uint8_t const* report, uint16_t len) {
    if (len < 8) {
        tuh_hid_receive_report(dev_addr, instance);
        return;
    }

    // Modifier delta
    uint8_t mod_diff = s_prev[0] ^ report[0];
    for (int b = 0; b < 8; b++) {
        if (mod_diff & (1 << b)) {
            omega_host_push_key(k_mod_hid[b], (report[0] >> b) & 1);
        }
    }

    // Teclas soltas (estavam no prev, não estão no atual)
    for (int i = 2; i < 8; i++) {
        uint8_t kc = s_prev[i];
        if (kc && !key_in_report(report, kc)) {
            omega_host_push_key(kc, 0);
        }
    }

    // Teclas pressionadas (estão no atual, não estavam no prev)
    for (int i = 2; i < 8; i++) {
        uint8_t kc = report[i];
        if (kc && !key_in_report(s_prev, kc)) {
            omega_host_push_key(kc, 1);
        }
    }

    // Salva relatório atual e solicita o próximo
    for (int i = 0; i < 8; i++) s_prev[i] = report[i];
    tuh_hid_receive_report(dev_addr, instance);
}
