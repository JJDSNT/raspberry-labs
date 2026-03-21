// lib/tinyusb/src/portable/synopsys/dwc2/dwc2_bcm.h  (substituído por dwc2_raspi3.h)
//
// Configuração do DWC2 para o Raspberry Pi 3 (BCM2837).
// Arquivo autossuficiente — sem dependências de SDK externo.
//

#ifndef DWC2_RASPI3_H_
#define DWC2_RASPI3_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include "dwc2_type.h"

// ---------------------------------------------------------------------------
// Endereço base e configuração do DWC2 no BCM2837
// ---------------------------------------------------------------------------
#define DWC2_EP_MAX       8
#define DWC2_FIFO_SIZE    16384

static const dwc2_controller_t _dwc2_controller[] = {
    {
        .reg_base     = 0x3F980000UL,
        .irqnum       = 0,
        .ep_count     = DWC2_EP_MAX,
        .ep_fifo_size = DWC2_FIFO_SIZE,
    }
};

// ---------------------------------------------------------------------------
// Cache — implementadas em hal_dwc2.c
// ---------------------------------------------------------------------------
extern void hcd_dwc2_dcache_clean(void *addr, uint32_t size);
extern void hcd_dwc2_dcache_invalidate(void *addr, uint32_t size);
extern void hcd_dwc2_dcache_clean_invalidate(void *addr, uint32_t size);

// Macros usadas pelo dwc2_common.h
#define dcache_clean(_addr, _size)            hcd_dwc2_dcache_clean((_addr), (_size))
#define dcache_invalidate(_addr, _size)       hcd_dwc2_dcache_invalidate((_addr), (_size))
#define dcache_clean_invalidate(_addr, _size) hcd_dwc2_dcache_clean_invalidate((_addr), (_size))

// Funções usadas pelo hcd_dwc2.c (linhas 150, 160)
static inline bool dwc2_dcache_clean(void const* addr, uint32_t data_size) {
    hcd_dwc2_dcache_clean((void*)addr, data_size);
    return true;
}

static inline bool dwc2_dcache_clean_invalidate(void const* addr, uint32_t data_size) {
    hcd_dwc2_dcache_clean_invalidate((void*)addr, data_size);
    return true;
}

static inline bool dwc2_dcache_invalidate(void const* addr, uint32_t data_size) {
    hcd_dwc2_dcache_invalidate((void*)addr, data_size);
    return true;
}

// ---------------------------------------------------------------------------
// dwc2_int_set — habilita/desabilita IRQ USB
// Chamado por hcd_dwc2.c linhas 461 e 466.
// No Pi 3 gerenciamos via VIC — por ora no-op pois a IRQ é
// habilitada globalmente durante init.
// ---------------------------------------------------------------------------
TU_ATTR_ALWAYS_INLINE
static inline void dwc2_int_set(uint8_t rhport, tusb_role_t role, bool enabled) {
    (void) rhport;
    (void) role;
    (void) enabled;
    // TODO: habilitar/desabilitar bit USB no VIC via kernel
    // Por ora deixamos a IRQ sempre habilitada após init
}

// Macros de compatibilidade usadas por outros headers
#define dwc2_dcd_int_enable(_rhport)  dwc2_int_set(_rhport, TUSB_ROLE_DEVICE, true)
#define dwc2_dcd_int_disable(_rhport) dwc2_int_set(_rhport, TUSB_ROLE_DEVICE, false)

// ---------------------------------------------------------------------------
// Remote wakeup delay
// ---------------------------------------------------------------------------
extern void hcd_dwc2_delay_us(uint32_t us);

static inline void dwc2_remote_wakeup_delay(void) {
    hcd_dwc2_delay_us(1000);
}

// ---------------------------------------------------------------------------
// PHY — interno no BCM2837, sem configuração necessária
// ---------------------------------------------------------------------------
static inline void dwc2_phy_init(dwc2_regs_t *dwc2, uint8_t hs_phy_type) {
    (void) dwc2; (void) hs_phy_type;
}

static inline void dwc2_phy_deinit(dwc2_regs_t *dwc2, uint8_t hs_phy_type) {
    (void) dwc2; (void) hs_phy_type;
}

static inline void dwc2_phy_update(dwc2_regs_t *dwc2, uint8_t hs_phy_type) {
    (void) dwc2; (void) hs_phy_type;
}

#ifdef __cplusplus
}
#endif

#endif // DWC2_RASPI3_H_