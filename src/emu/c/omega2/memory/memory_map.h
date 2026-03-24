#ifndef MEMORY_MAP_H
#define MEMORY_MAP_H

#include <stdint.h>

/*
 * Notes:
 *
 * - AROS requires Fast RAM to boot correctly.
 * - In this implementation, Fast RAM is currently provided via
 *   ZII expansion space and/or custom RAM regions.
 *
 * - CIA and Chipset state are allocated outside the emulated
 *   address space (see Memory.c).
 */

/*
 * Address space region sizes (in bytes)
 */

enum {
    CHIP_RAM_SIZE            = 2097152,   /* 2 MB */
    ZII_SPACE_SIZE           = 8388608,   /* 8 MB */

    /*
     * 0xA00000 - 0xBEFFFF
     * Available as RAM (replaces old CIA/Chipstate areas)
     */
    AROS_CUSTOM_RAM_SIZE     = 2031616,

    CIA_SPACE_SIZE           = 65536,
    SLOW_RAM_SIZE            = 1572864,
    RESERVED2_SIZE           = 131072,

    IDE_CONTROLLER_SIZE      = 131072,
    REAL_TIME_CLOCK_SIZE     = 65536,
    RESERVED3_SIZE           = 65536,

    MAINBOARD_RESOURCES_SIZE = 65536,
    CUSTOM_CHIP_REGS_SIZE    = 65536,
    AUTOCONFIG_SPACE_SIZE    = 1048576,

    EXTENDED_ROM_SIZE        = 524288,
    KICKSTART_ROM_SIZE       = 524288
};

/*
 * Full emulated memory layout
 */

typedef struct
{
    uint8_t chip_ram[CHIP_RAM_SIZE];
    uint8_t zii_space[ZII_SPACE_SIZE];

    uint8_t aros_custom_ram[AROS_CUSTOM_RAM_SIZE];

    uint8_t cia_space[CIA_SPACE_SIZE];
    uint8_t slow_ram[SLOW_RAM_SIZE];
    uint8_t reserved2[RESERVED2_SIZE];

    uint8_t ide_controller[IDE_CONTROLLER_SIZE];
    uint8_t real_time_clock[REAL_TIME_CLOCK_SIZE];
    uint8_t reserved3[RESERVED3_SIZE];

    uint8_t mainboard_resources[MAINBOARD_RESOURCES_SIZE];
    uint8_t custom_chip_registers[CUSTOM_CHIP_REGS_SIZE];
    uint8_t autoconfig_space[AUTOCONFIG_SPACE_SIZE];

    uint8_t extended_rom[EXTENDED_ROM_SIZE];
    uint8_t kickstart_rom[KICKSTART_ROM_SIZE];

} MemoryMap;

#endif /* MEMORY_MAP_H */