#ifndef OMEGA2_STORAGE_HDFBACKEND_H
#define OMEGA2_STORAGE_HDFBACKEND_H

#include <stdint.h>
#include <stddef.h>
#include <stdio.h>

#define HDF_BLOCK_SIZE 512

typedef struct HdfBackend {
    FILE* fp;
    uint64_t size_bytes;
    uint64_t total_blocks;
} HdfBackend;

/*
 * Abre uma imagem HDF/RAW em modo somente leitura.
 * Retorna 0 em caso de sucesso.
 */
int hdf_open(HdfBackend* hdf, const char* path);

/*
 * Fecha o backend, se estiver aberto.
 */
void hdf_close(HdfBackend* hdf);

/*
 * Lê 'count' blocos de 512 bytes a partir de 'lba' para 'buffer'.
 * Retorna 0 em caso de sucesso.
 */
int hdf_read(HdfBackend* hdf, uint64_t lba, uint32_t count, void* buffer);

#endif