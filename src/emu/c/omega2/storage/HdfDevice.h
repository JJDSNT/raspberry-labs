#ifndef OMEGA2_STORAGE_HDFDEVICE_H
#define OMEGA2_STORAGE_HDFDEVICE_H

#include <stdint.h>
#include "HdfBackend.h"

/*
 * MMIO layout relativo ao base address:
 *
 * 0x000 STATUS   (ro, 32)
 * 0x004 COMMAND  (wo, 32)
 * 0x008 LBA_LO   (rw, 32)
 * 0x00C LBA_HI   (rw, 32)
 * 0x010 COUNT    (rw, 32)
 * 0x014 ERROR    (ro, 32)
 * 0x100 BUFFER   (ro, 512 bytes)
 *
 * MVP:
 * - somente leitura
 * - somente 1 bloco por comando
 */

#define HDF_MMIO_SIZE        0x1000
#define HDF_MMIO_BUFFER_SIZE 512

#define HDF_REG_STATUS   0x000
#define HDF_REG_COMMAND  0x004
#define HDF_REG_LBA_LO   0x008
#define HDF_REG_LBA_HI   0x00C
#define HDF_REG_COUNT    0x010
#define HDF_REG_ERROR    0x014
#define HDF_REG_BUFFER   0x100

#define HDF_STATUS_READY  (1u << 0)
#define HDF_STATUS_BUSY   (1u << 1)
#define HDF_STATUS_ERROR  (1u << 2)

#define HDF_CMD_NONE      0
#define HDF_CMD_READ      1

typedef struct HdfDevice {
    HdfBackend backend;

    uint32_t status;
    uint32_t command;
    uint32_t lba_lo;
    uint32_t lba_hi;
    uint32_t count;
    uint32_t error_code;

    uint8_t buffer[HDF_MMIO_BUFFER_SIZE];
} HdfDevice;

void hdf_device_reset(HdfDevice* dev);

int hdf_device_init(HdfDevice* dev, const char* path);
void hdf_device_shutdown(HdfDevice* dev);

uint32_t hdf_device_read32(HdfDevice* dev, uint32_t offset);
uint16_t hdf_device_read16(HdfDevice* dev, uint32_t offset);
uint8_t  hdf_device_read8(HdfDevice* dev, uint32_t offset);

void hdf_device_write32(HdfDevice* dev, uint32_t offset, uint32_t value);

#endif