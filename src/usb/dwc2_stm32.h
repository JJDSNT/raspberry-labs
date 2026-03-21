// src/usb/dwc2_stm32.h
//
// Stub que redireciona o include do TinyUSB para nosso header do Pi 3.
// O dwc2_common.h inclui "dwc2_stm32.h" quando TUP_USBIP_DWC2_STM32
// está definido. Como src/usb/ vem primeiro no include path, este
// arquivo é encontrado antes do original do TinyUSB.
//

#include "dwc2_raspi3.h"