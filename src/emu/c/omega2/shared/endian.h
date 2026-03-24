#ifndef ENDIAN_H
#define ENDIAN_H

#include <stdint.h>

/*
 * Byte swap helpers
 */

static inline uint16_t bswap16(uint16_t value)
{
    return (uint16_t)((value >> 8) | (value << 8));
}

static inline uint32_t bswap32(uint32_t value)
{
    return ((value & 0x000000FFu) << 24) |
           ((value & 0x0000FF00u) <<  8) |
           ((value & 0x00FF0000u) >>  8) |
           ((value & 0xFF000000u) >> 24);
}

/*
 * Big-endian reads from arbitrary byte buffers
 */

static inline uint8_t read_u8(const uint8_t *base, uint32_t addr)
{
    return base[addr];
}

static inline uint16_t read_be16(const uint8_t *base, uint32_t addr)
{
    return ((uint16_t)base[addr] << 8) |
           ((uint16_t)base[addr + 1]);
}

static inline uint32_t read_be32(const uint8_t *base, uint32_t addr)
{
    return ((uint32_t)base[addr]     << 24) |
           ((uint32_t)base[addr + 1] << 16) |
           ((uint32_t)base[addr + 2] <<  8) |
           ((uint32_t)base[addr + 3]);
}

/*
 * Little-endian reads from arbitrary byte buffers
 */

static inline uint16_t read_le16(const uint8_t *base, uint32_t addr)
{
    return ((uint16_t)base[addr]) |
           ((uint16_t)base[addr + 1] << 8);
}

static inline uint32_t read_le32(const uint8_t *base, uint32_t addr)
{
    return ((uint32_t)base[addr]) |
           ((uint32_t)base[addr + 1] << 8) |
           ((uint32_t)base[addr + 2] << 16) |
           ((uint32_t)base[addr + 3] << 24);
}

/*
 * Big-endian writes to arbitrary byte buffers
 */

static inline void write_u8(uint8_t *base, uint32_t addr, uint8_t value)
{
    base[addr] = value;
}

static inline void write_be16(uint8_t *base, uint32_t addr, uint16_t value)
{
    base[addr]     = (uint8_t)(value >> 8);
    base[addr + 1] = (uint8_t)(value);
}

static inline void write_be32(uint8_t *base, uint32_t addr, uint32_t value)
{
    base[addr]     = (uint8_t)(value >> 24);
    base[addr + 1] = (uint8_t)(value >> 16);
    base[addr + 2] = (uint8_t)(value >> 8);
    base[addr + 3] = (uint8_t)(value);
}

/*
 * Little-endian writes to arbitrary byte buffers
 */

static inline void write_le16(uint8_t *base, uint32_t addr, uint16_t value)
{
    base[addr]     = (uint8_t)(value);
    base[addr + 1] = (uint8_t)(value >> 8);
}

static inline void write_le32(uint8_t *base, uint32_t addr, uint32_t value)
{
    base[addr]     = (uint8_t)(value);
    base[addr + 1] = (uint8_t)(value >> 8);
    base[addr + 2] = (uint8_t)(value >> 16);
    base[addr + 3] = (uint8_t)(value >> 24);
}

#endif /* ENDIAN_H */