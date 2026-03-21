// src/usb/tusb_config.h

#ifndef _TUSB_CONFIG_H_
#define _TUSB_CONFIG_H_

#ifdef __cplusplus
extern "C" {
#endif

// MCU e OS
#define CFG_TUSB_MCU      OPT_MCU_BCM2837
#define CFG_TUSB_OS       OPT_OS_NONE
#define CFG_TUSB_DEBUG    0

// Driver DWC2 — ativa hcd_dwc2.c
// dwc2_common.h vai incluir dwc2_bcm.h (que substituímos pelo nosso)
// #define TUP_USBIP_DWC2    1

// Modo de transferência — slave (polling FIFO, mais simples para início)
#define CFG_TUH_DWC2_SLAVE_ENABLE  1
#define CFG_TUH_DWC2_ENDPOINT_MAX  16

// Host
#define CFG_TUSB_RHPORT0_MODE (OPT_MODE_HOST | OPT_MODE_FULL_SPEED)
#define CFG_TUH_ENABLED       1
#define CFG_TUH_DEVICE_MAX    4
#define CFG_TUH_HUB           1
#define CFG_TUH_HID           4
#define CFG_TUH_MSC           1

// DCache
#define CFG_TUH_MEM_DCACHE_ENABLE  1

// Memória
#define CFG_TUSB_MEM_SIZE     4096
#define CFG_TUSB_MEM_ALIGN    TU_ATTR_ALIGNED(4)

#ifdef __cplusplus
}
#endif

#endif