#include "HdfBackend.h"

#include <string.h>

/*
 * Backend real implementado no lado Rust.
 *
 * Contrato esperado:
 *
 *   omega_hdf_open(path, &size_bytes) -> handle opaco ou NULL
 *   omega_hdf_read(handle, offset, dst, size) -> 0 sucesso, <0 erro
 *   omega_hdf_close(handle)
 *
 * O Rust pode implementar isso usando FAT32 + SD card.
 */

extern HdfHandle omega_hdf_open(const char* path, uint64_t* size_bytes);
extern int omega_hdf_read(HdfHandle handle, uint64_t offset, void* buffer, uint32_t size);
extern void omega_hdf_close(HdfHandle handle);

int hdf_open(HdfBackend* hdf, const char* path)
{
    uint64_t size_bytes = 0;
    HdfHandle handle;

    if (!hdf || !path) return -1;

    memset(hdf, 0, sizeof(*hdf));

    handle = omega_hdf_open(path, &size_bytes);
    if (!handle) return -2;

    if (size_bytes < HDF_BLOCK_SIZE) {
        omega_hdf_close(handle);
        return -3;
    }

    if ((size_bytes % HDF_BLOCK_SIZE) != 0) {
        omega_hdf_close(handle);
        return -4;
    }

    hdf->handle = handle;
    hdf->size_bytes = size_bytes;
    hdf->total_blocks = size_bytes / HDF_BLOCK_SIZE;

    return 0;
}

void hdf_close(HdfBackend* hdf)
{
    if (!hdf) return;

    if (hdf->handle) {
        omega_hdf_close(hdf->handle);
    }

    hdf->handle = (HdfHandle)0;
    hdf->size_bytes = 0;
    hdf->total_blocks = 0;
}

int hdf_read(HdfBackend* hdf, uint64_t lba, uint32_t count, void* buffer)
{
    uint64_t offset;
    uint64_t bytes_total;

    if (!hdf || !hdf->handle || !buffer) return -1;
    if (count == 0) return 0;
    if (lba >= hdf->total_blocks) return -2;
    if ((lba + count) > hdf->total_blocks) return -3;

    offset = lba * HDF_BLOCK_SIZE;
    bytes_total = (uint64_t)count * HDF_BLOCK_SIZE;

    /*
     * O backend Rust recebe tamanho em u32.
     * Para o MVP isso é suficiente; count costuma ser pequeno.
     */
    if (bytes_total > 0xFFFFFFFFu) return -4;

    return omega_hdf_read(hdf->handle, offset, buffer, (uint32_t)bytes_total);
}