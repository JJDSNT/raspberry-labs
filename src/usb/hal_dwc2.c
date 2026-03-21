// src/usb/hal_dwc2.c
//
// HAL do DWC2 para o Raspberry Pi 3 (BCM2837).
//

#include <stdint.h>
#include <stddef.h>
#include "tusb.h"

// ---------------------------------------------------------------------------
// Funções externas do kernel Rust
// ---------------------------------------------------------------------------
extern uint64_t kernel_time_ms(void);
extern void     kernel_delay_us(uint32_t us);
extern void     kernel_cache_flush(uintptr_t start, uintptr_t end);
extern int      mailbox_usb_power(int on);

// ---------------------------------------------------------------------------
// Endereço base do DWC2 no BCM2837
// ---------------------------------------------------------------------------
#define DWC2_BASE   0x3F980000UL

// ---------------------------------------------------------------------------
// tusb_time_millis_api — timestamp em ms para timeouts do TinyUSB
// Chamado por usbh.c para gerenciar timeouts de transferência.
// ---------------------------------------------------------------------------
uint32_t tusb_time_millis_api(void) {
    return (uint32_t)kernel_time_ms();
}

// ---------------------------------------------------------------------------
// hcd_dwc2_reg_base — endereço base do DWC2
// ---------------------------------------------------------------------------
uintptr_t hcd_dwc2_reg_base(uint8_t rhport) {
    (void)rhport;
    return DWC2_BASE;
}

// ---------------------------------------------------------------------------
// hcd_dwc2_init — power on via mailbox
// ---------------------------------------------------------------------------
bool hcd_dwc2_init(uint8_t rhport) {
    (void)rhport;
    return mailbox_usb_power(1) == 0;
}

// ---------------------------------------------------------------------------
// hcd_int_handler — forward da IRQ USB para o TinyUSB
// Chamado pelo nosso irq.rs quando a IRQ USB dispara no VIC.
// ---------------------------------------------------------------------------
// void hcd_int_handler(uint8_t rhport, bool in_isr) {
//    tusb_int_handler(rhport, in_isr);
//}

// ---------------------------------------------------------------------------
// Cache — coerência entre CPU e DMA do DWC2
// ---------------------------------------------------------------------------
void hcd_dwc2_dcache_clean(void *addr, uint32_t size) {
    kernel_cache_flush((uintptr_t)addr, (uintptr_t)addr + size);
}

void hcd_dwc2_dcache_invalidate(void *addr, uint32_t size) {
    kernel_cache_flush((uintptr_t)addr, (uintptr_t)addr + size);
}

void hcd_dwc2_dcache_clean_invalidate(void *addr, uint32_t size) {
    kernel_cache_flush((uintptr_t)addr, (uintptr_t)addr + size);
}

// ---------------------------------------------------------------------------
// Delay
// ---------------------------------------------------------------------------
void hcd_dwc2_delay_us(uint32_t us) {
    kernel_delay_us(us);
}

// ---------------------------------------------------------------------------
// Callbacks do TinyUSB — chamados quando eventos USB ocorrem
//
// Estas funções são chamadas pelo stack do TinyUSB e devem ser
// implementadas pela aplicação. Por enquanto só logamos os eventos.
// ---------------------------------------------------------------------------

// Chamado quando um dispositivo HID envia um relatório
TU_ATTR_WEAK void tuh_hid_report_received_cb(uint8_t dev_addr, uint8_t instance,
                                              uint8_t const *report, uint16_t len) {
    (void)dev_addr;
    (void)instance;
    (void)report;
    (void)len;
    // TODO: processar relatório HID (teclado, mouse, gamepad)
}

// Chamado quando um dispositivo HID é montado
TU_ATTR_WEAK void tuh_hid_mount_cb(uint8_t dev_addr, uint8_t instance,
                                    uint8_t const *desc_report, uint16_t desc_len) {
    (void)dev_addr;
    (void)instance;
    (void)desc_report;
    (void)desc_len;
    // TODO: identificar tipo de dispositivo e iniciar recepção
}

// Chamado quando um dispositivo HID é desmontado
TU_ATTR_WEAK void tuh_hid_umount_cb(uint8_t dev_addr, uint8_t instance) {
    (void)dev_addr;
    (void)instance;
}

// Chamado quando um dispositivo MSC é montado
TU_ATTR_WEAK void tuh_msc_mount_cb(uint8_t dev_addr) {
    (void)dev_addr;
    // TODO: inicializar filesystem
}

// Chamado quando um dispositivo MSC é desmontado
TU_ATTR_WEAK void tuh_msc_umount_cb(uint8_t dev_addr) {
    (void)dev_addr;
}