#ifndef OMEGA2_STORAGE_HDFBACKEND_H
#define OMEGA2_STORAGE_HDFBACKEND_H

#include <stdint.h>
#include <stddef.h>

#define HDF_BLOCK_SIZE 512

/*
 * Handle opaco para o backend.
 * A implementação real (Rust/FS/SD) decide o que isso significa.
 */
typedef void* HdfHandle;

typedef struct HdfBackend {
    HdfHandle handle;
    uint64_t size_bytes;
    uint64_t total_blocks;
} HdfBackend;

/*
 * Abre uma imagem HDF/RAW em modo somente leitura.
 *
 * 'path' é um nome de arquivo (ex: "workbench.hdf")
 *
 * Retorna:
 *   0  sucesso
 *  <0  erro
 */
int hdf_open(HdfBackend* hdf, const char* path);

/*
 * Fecha o backend, se estiver aberto.
 */
void hdf_close(HdfBackend* hdf);

/*
 * Lê 'count' blocos de 512 bytes a partir de 'lba' para 'buffer'.
 *
 * Requisitos:
 * - buffer deve ter count * 512 bytes
 *
 * Retorna:
 *   0  sucesso
 *  <0  erro
 */
int hdf_read(HdfBackend* hdf, uint64_t lba, uint32_t count, void* buffer);

#endif