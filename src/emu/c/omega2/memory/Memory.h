#ifndef MEMORY_H
#define MEMORY_H

#include <stdint.h>

#include "memory_map.h"

/*
 * Core memory state
 */

extern uint8_t *RAM24bit;
extern const char *regNames[];

/*
 * Memory access sizes
 */

typedef enum
{
    MEM_SIZE_BYTE = 0,
    MEM_SIZE_WORD = 1,
    MEM_SIZE_LONG = 2
} MemoryAccessSize;

/*
 * Raw buffer helpers
 *
 * These functions operate on arbitrary byte buffers and are safe with respect
 * to alignment and strict aliasing. They also make endianness explicit.
 */

static inline uint8_t read_u8(const uint8_t *base, uint32_t addr)
{
    return base[addr];
}

static inline void write_u8(uint8_t *base, uint32_t addr, uint8_t value)
{
    base[addr] = value;
}

static inline uint16_t read_be16(const uint8_t *base, uint32_t addr)
{
    return ((uint16_t)base[addr] << 8) |
           ((uint16_t)base[addr + 1]);
}

static inline void write_be16(uint8_t *base, uint32_t addr, uint16_t value)
{
    base[addr]     = (uint8_t)(value >> 8);
    base[addr + 1] = (uint8_t)(value);
}

static inline uint32_t read_be32(const uint8_t *base, uint32_t addr)
{
    return ((uint32_t)base[addr]     << 24) |
           ((uint32_t)base[addr + 1] << 16) |
           ((uint32_t)base[addr + 2] <<  8) |
           ((uint32_t)base[addr + 3]);
}

static inline void write_be32(uint8_t *base, uint32_t addr, uint32_t value)
{
    base[addr]     = (uint8_t)(value >> 24);
    base[addr + 1] = (uint8_t)(value >> 16);
    base[addr + 2] = (uint8_t)(value >>  8);
    base[addr + 3] = (uint8_t)(value);
}

/*
 * Direct access helpers for the main emulated address space
 */

static inline uint8_t mem_read_u8(uint32_t addr)
{
    return read_u8(RAM24bit, addr);
}

static inline void mem_write_u8(uint32_t addr, uint8_t value)
{
    write_u8(RAM24bit, addr, value);
}

static inline uint16_t mem_read_u16(uint32_t addr)
{
    return read_be16(RAM24bit, addr);
}

static inline void mem_write_u16(uint32_t addr, uint16_t value)
{
    write_be16(RAM24bit, addr, value);
}

static inline uint32_t mem_read_u32(uint32_t addr)
{
    return read_be32(RAM24bit, addr);
}

static inline void mem_write_u32(uint32_t addr, uint32_t value)
{
    write_be32(RAM24bit, addr, value);
}

/*
 * Legacy compatibility macros
 *
 * These can be removed later once the codebase has been migrated to the
 * mem_read_u* and mem_write_u* helpers directly.
 */

#define READBYTE(address)          mem_read_u8((uint32_t)(address))
#define WRITEBYTE(address, value)  mem_write_u8((uint32_t)(address), (uint8_t)(value))
#define READWORD(address)          mem_read_u16((uint32_t)(address))
#define WRITEWORD(address, value)  mem_write_u16((uint32_t)(address), (uint16_t)(value))
#define READLONG(address)          mem_read_u32((uint32_t)(address))
#define WRITELONG(address, value)  mem_write_u32((uint32_t)(address), (uint32_t)(value))

/*
 * CPU / bus interface
 */

uint32_t cpu_read_byte(uint32_t address);
uint32_t cpu_read_word(uint32_t address);
uint32_t cpu_read_long(uint32_t address);

void cpu_write_byte(uint32_t address, uint32_t value);
void cpu_write_word(uint32_t address, uint32_t value);
void cpu_write_long(uint32_t address, uint32_t value);

/*
 * CPU integration hooks
 */

void cpu_pulse_reset(void);
void cpu_set_fc(uint32_t fc);
int  cpu_irq_ack(int level);
void cpu_instr_callback(void);

/*
 * Disassembler helpers
 */

uint32_t cpu_read_word_dasm(uint32_t address);
uint32_t cpu_read_long_dasm(uint32_t address);

/*
 * Memory lifecycle
 */

MemoryMap *memory_init(uint32_t fast_ram_size);
void memory_reset(void);

/*
 * Generic memory access helpers
 */

uint32_t memory_read(uint32_t address, MemoryAccessSize size);
void memory_write(uint32_t address, MemoryAccessSize size, uint32_t value);

/*
 * Diagnostics
 *
 * Consider moving this elsewhere later if you want Memory.h to remain focused
 * purely on memory and bus responsibilities.
 */

void printCPUContext(void);

#endif /* MEMORY_H */