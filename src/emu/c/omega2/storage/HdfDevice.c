#include "HdfDevice.h"

#include <string.h>
#include <stdio.h>

static uint64_t hdf_device_get_lba(const HdfDevice* dev) {
    return ((uint64_t)dev->lba_hi << 32) | (uint64_t)dev->lba_lo;
}

void hdf_device_reset(HdfDevice* dev) {
    if (!dev) return;

    dev->status = 0;
    dev->command = HDF_CMD_NONE;
    dev->lba_lo = 0;
    dev->lba_hi = 0;
    dev->count = 1;
    dev->error_code = 0;
    memset(dev->buffer, 0, sizeof(dev->buffer));
}

static void hdf_device_set_error(HdfDevice* dev, uint32_t error_code) {
    dev->error_code = error_code;
    dev->status = HDF_STATUS_ERROR;
    dev->command = HDF_CMD_NONE;
}

static void hdf_device_exec_read(HdfDevice* dev) {
    uint64_t lba;
    uint32_t count;
    int rc;

    lba = hdf_device_get_lba(dev);
    count = dev->count;

    if (count == 0) count = 1;

    /* MVP: apenas 1 bloco por comando */
    if (count != 1) {
        hdf_device_set_error(dev, 1001);
        return;
    }

    /* debug opcional */
    /* printf("[HDF] READ lba=%llu count=%u\n", (unsigned long long)lba, count); */

    rc = hdf_read(&dev->backend, lba, 1, dev->buffer);
    if (rc != 0) {
        hdf_device_set_error(dev, (uint32_t)(2000 - rc));
        return;
    }

    dev->status = HDF_STATUS_READY;
    dev->error_code = 0;
    dev->command = HDF_CMD_NONE;
}

static void hdf_device_exec(HdfDevice* dev) {
    dev->status = HDF_STATUS_BUSY;
    dev->error_code = 0;

    switch (dev->command) {
        case HDF_CMD_READ:
            hdf_device_exec_read(dev);
            break;

        default:
            hdf_device_set_error(dev, 9999);
            break;
    }
}

int hdf_device_init(HdfDevice* dev, const char* path) {
    int rc;

    if (!dev) return -1;

    memset(dev, 0, sizeof(*dev));
    hdf_device_reset(dev);

    rc = hdf_open(&dev->backend, path);
    if (rc != 0) return rc;

    dev->status = HDF_STATUS_READY;
    return 0;
}

void hdf_device_shutdown(HdfDevice* dev) {
    if (!dev) return;

    hdf_close(&dev->backend);
    hdf_device_reset(dev);
}

uint32_t hdf_device_read32(HdfDevice* dev, uint32_t offset) {
    if (!dev) return 0;

    switch (offset) {
        case HDF_REG_STATUS:  return dev->status;
        case HDF_REG_COMMAND: return dev->command;
        case HDF_REG_LBA_LO:  return dev->lba_lo;
        case HDF_REG_LBA_HI:  return dev->lba_hi;
        case HDF_REG_COUNT:   return dev->count;
        case HDF_REG_ERROR:   return dev->error_code;

        default:
            if (offset >= HDF_REG_BUFFER &&
                offset + 3 < HDF_REG_BUFFER + HDF_MMIO_BUFFER_SIZE) {
                uint32_t i = offset - HDF_REG_BUFFER;
                return ((uint32_t)dev->buffer[i + 0] << 24) |
                       ((uint32_t)dev->buffer[i + 1] << 16) |
                       ((uint32_t)dev->buffer[i + 2] << 8)  |
                       ((uint32_t)dev->buffer[i + 3]);
            }
            return 0;
    }
}

uint16_t hdf_device_read16(HdfDevice* dev, uint32_t offset) {
    if (!dev) return 0;

    if (offset >= HDF_REG_BUFFER &&
        offset + 1 < HDF_REG_BUFFER + HDF_MMIO_BUFFER_SIZE) {
        uint32_t i = offset - HDF_REG_BUFFER;
        return (uint16_t)(((uint16_t)dev->buffer[i + 0] << 8) |
                          ((uint16_t)dev->buffer[i + 1]));
    }

    {
        uint32_t v = hdf_device_read32(dev, offset & ~3u);
        if ((offset & 2u) == 0)
            return (uint16_t)(v >> 16);
        return (uint16_t)(v & 0xFFFFu);
    }
}

uint8_t hdf_device_read8(HdfDevice* dev, uint32_t offset) {
    if (!dev) return 0;

    if (offset >= HDF_REG_BUFFER &&
        offset < HDF_REG_BUFFER + HDF_MMIO_BUFFER_SIZE) {
        return dev->buffer[offset - HDF_REG_BUFFER];
    }

    {
        uint32_t v = hdf_device_read32(dev, offset & ~3u);
        switch (offset & 3u) {
            case 0: return (uint8_t)(v >> 24);
            case 1: return (uint8_t)(v >> 16);
            case 2: return (uint8_t)(v >> 8);
            default: return (uint8_t)(v);
        }
    }
}

void hdf_device_write32(HdfDevice* dev, uint32_t offset, uint32_t value) {
    if (!dev) return;

    switch (offset) {
        case HDF_REG_COMMAND:
            dev->command = value;
            hdf_device_exec(dev);
            break;

        case HDF_REG_LBA_LO:
            dev->lba_lo = value;
            break;

        case HDF_REG_LBA_HI:
            dev->lba_hi = value;
            break;

        case HDF_REG_COUNT:
            dev->count = value;
            break;

        default:
            break;
    }
}