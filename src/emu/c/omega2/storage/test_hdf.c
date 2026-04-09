#include <stdio.h>
#include <stdint.h>
#include <string.h>

#include "HdfBackend.h"
#include "HdfDevice.h"

static uint32_t be32(const uint8_t* p) {
    return ((uint32_t)p[0] << 24) |
           ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] << 8)  |
           ((uint32_t)p[3]);
}

static void dump_hex(const uint8_t* p, int size) {
    int i;

    for (i = 0; i < size; i++) {
        if ((i % 16) == 0) printf("%04x: ", i);
        printf("%02x ", p[i]);
        if ((i % 16) == 15) printf("\n");
    }

    if ((size % 16) != 0) printf("\n");
}

static int is_rdsk(const uint8_t* block) {
    return be32(block) == 0x5244534bU; /* 'RDSK' */
}

/*
 * Retorna:
 *   >= 0 : LBA onde encontrou RDSK
 *   -1   : nao encontrou
 *   -2   : falha ao abrir backend
 */
static int scan_rdb_backend(const char* path) {
    HdfBackend hdf;
    uint8_t block[HDF_BLOCK_SIZE];
    int rc;
    int i;

    rc = hdf_open(&hdf, path);
    printf("hdf_open(%s) -> %d\n", path, rc);
    if (rc != 0) return -2;

    printf("size_bytes=%llu total_blocks=%llu\n",
           (unsigned long long)hdf.size_bytes,
           (unsigned long long)hdf.total_blocks);

    for (i = 0; i < 16; i++) {
        rc = hdf_read(&hdf, (uint64_t)i, 1, block);
        if (rc != 0) {
            printf("read lba %d failed rc=%d\n", i, rc);
            continue;
        }

        printf("lba=%d sig=%c%c%c%c\n",
               i,
               block[0], block[1], block[2], block[3]);

        if (is_rdsk(block)) {
            printf("RDSK encontrado em lba=%d\n", i);
            dump_hex(block, 64);
            hdf_close(&hdf);
            return i;
        }
    }

    printf("RDSK nao encontrado em 0..15\n");
    hdf_close(&hdf);
    return -1;
}

static void test_mmio_device(const char* path, uint32_t lba) {
    HdfDevice dev;
    int rc;
    int i;

    rc = hdf_device_init(&dev, path);
    printf("hdf_device_init(%s) -> %d\n", path, rc);
    if (rc != 0) return;

    printf("MMIO READ lba=%u\n", lba);

    hdf_device_write32(&dev, HDF_REG_LBA_LO, lba);
    hdf_device_write32(&dev, HDF_REG_LBA_HI, 0);
    hdf_device_write32(&dev, HDF_REG_COUNT, 1);
    hdf_device_write32(&dev, HDF_REG_COMMAND, HDF_CMD_READ);

    printf("status=%08x error=%u\n",
           hdf_device_read32(&dev, HDF_REG_STATUS),
           hdf_device_read32(&dev, HDF_REG_ERROR));

    printf("sig=%c%c%c%c\n",
           hdf_device_read8(&dev, HDF_REG_BUFFER + 0),
           hdf_device_read8(&dev, HDF_REG_BUFFER + 1),
           hdf_device_read8(&dev, HDF_REG_BUFFER + 2),
           hdf_device_read8(&dev, HDF_REG_BUFFER + 3));

    for (i = 0; i < 64; i++) {
        if ((i % 16) == 0) printf("%04x: ", i);
        printf("%02x ", hdf_device_read8(&dev, HDF_REG_BUFFER + i));
        if ((i % 16) == 15) printf("\n");
    }

    hdf_device_shutdown(&dev);
}

int main(int argc, char** argv) {
    const char* path;
    int rdsk_lba;

    if (argc < 2) {
        printf("uso: %s arquivo.hdf\n", argv[0]);
        return 1;
    }

    path = argv[1];

    printf("== scan backend ==\n");
    rdsk_lba = scan_rdb_backend(path);

    printf("\n== teste mmio bloco 0 ==\n");
    test_mmio_device(path, 0);

    if (rdsk_lba >= 0) {
        printf("\n== teste mmio bloco RDSK ==\n");
        test_mmio_device(path, (uint32_t)rdsk_lba);
    }

    return 0;
}