#include "HdfBackend.h"

#include <string.h>

int hdf_open(HdfBackend* hdf, const char* path) {
    long size;

    if (!hdf || !path) return -1;

    memset(hdf, 0, sizeof(*hdf));

    hdf->fp = fopen(path, "rb");
    if (!hdf->fp) return -2;

    if (fseek(hdf->fp, 0, SEEK_END) != 0) {
        fclose(hdf->fp);
        hdf->fp = NULL;
        return -3;
    }

    size = ftell(hdf->fp);
    if (size < 0) {
        fclose(hdf->fp);
        hdf->fp = NULL;
        return -4;
    }

    if (fseek(hdf->fp, 0, SEEK_SET) != 0) {
        fclose(hdf->fp);
        hdf->fp = NULL;
        return -5;
    }

    if (size < HDF_BLOCK_SIZE) {
        fclose(hdf->fp);
        hdf->fp = NULL;
        return -6;
    }

    if ((size % HDF_BLOCK_SIZE) != 0) {
        fclose(hdf->fp);
        hdf->fp = NULL;
        return -7;
    }

    hdf->size_bytes = (uint64_t)size;
    hdf->total_blocks = hdf->size_bytes / HDF_BLOCK_SIZE;

    return 0;
}

void hdf_close(HdfBackend* hdf) {
    if (!hdf) return;

    if (hdf->fp) {
        fclose(hdf->fp);
    }

    hdf->fp = NULL;
    hdf->size_bytes = 0;
    hdf->total_blocks = 0;
}

int hdf_read(HdfBackend* hdf, uint64_t lba, uint32_t count, void* buffer) {
    uint64_t offset;
    size_t bytes_read;
    size_t bytes_total;

    if (!hdf || !hdf->fp || !buffer) return -1;
    if (count == 0) return 0;
    if (lba >= hdf->total_blocks) return -2;
    if ((lba + count) > hdf->total_blocks) return -3;

    offset = lba * HDF_BLOCK_SIZE;
    bytes_total = (size_t)count * HDF_BLOCK_SIZE;

    if (fseek(hdf->fp, (long)offset, SEEK_SET) != 0) return -4;

    bytes_read = fread(buffer, 1, bytes_total, hdf->fp);
    if (bytes_read != bytes_total) return -5;

    return 0;
}