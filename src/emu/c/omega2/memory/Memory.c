//
//  Memory.c
//  Omega2
//
//  Revised for bare-metal Raspberry Pi 3
//  Policy:
//  - external ROM from host/SD has priority
//  - built-in AROS is fallback only
//

#include "Memory.h"

#if defined(USE_AROS) && __has_include("aros_main.h")
#include "aros_main.h"
#include "aros_ext.h"
#define HAVE_AROS 1
#endif

#include "m68k.h"
#include "CIA.h"
#include "Floppy.h"

#include "omega_probe.h"
#include "omega_host.h"

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

/* -------------------------------------------------------------------------- */
/* Physical memory layout                                                      */
/* -------------------------------------------------------------------------- */

#define OMEGA_PHYS_ADDR          0x01000000UL   /* 16 MB */
#define CHIPSTATE_PHYS_ADDR      0x02400000UL   /* 36 MB */
#define CIASTATE_PHYS_ADDR       0x02410000UL   /* 36 MB + 64 KB */

/* -------------------------------------------------------------------------- */
/* Amiga 24-bit address space                                                  */
/* -------------------------------------------------------------------------- */

#define AMIGA_ADDR_MASK          0x00FFFFFFU

#define CHIP_RAM_START           0x000000U
#define CHIP_RAM_END             0x1FFFFFU

#define ZORRO2_START             0x200000U
#define ZORRO2_END               0x9FFFFFU

#define SLOW_RAM_START           0xA00000U
#define SLOW_RAM_END             0xBEFFFFU

#define CIA_START                0xBF0000U
#define CIA_END                  0xBFFFFFU

#define CHIP_MIRROR_START        0xC00000U
#define CHIP_MIRROR_END          0xD7FFFFU

#define CHIPREG_START            0xD80000U
#define CHIPREG_END              0xDFFFFFU

#define EXT_ROM_START            0xE00000U
#define EXT_ROM_END              0xE7FFFFU

#define AUTOCONFIG_START         0xE80000U
#define AUTOCONFIG_END           0xEFFFFFU

#define KICK_ROM_START           0xF00000U
#define KICK_ROM_MAIN_START      0xF80000U
#define KICK_ROM_BOOT_PC         0xF80002U
#define KICK_ROM_END             0xFFFFFFU

#define AROS_EXT_START           0xE00000U
#define AROS_MAIN_START          0xF80000U

#define AROS_BANK0_START         0xA80000U
#define AROS_BANK0_END           0xAFFFFFU
#define AROS_BANK1_START         0xB00000U
#define AROS_BANK1_END           0xB7FFFFU

#define CHIPREG_BASE             0xDFF000U
#define CHIPREG_ALIAS_MASK       0x01FFU
#define DMACON_OFFSET            0x009AU

#define CIA_ICR_A_ADDR           0xBFED01U
#define CIA_ICR_B_ADDR           0xBFDD00U

#define INTREQ_ADDR              0xDFF09CU
#define INTREQ_CLR_PORTS         0x0008U
#define INTREQ_CLR_EXTER         0x2000U

#define RTC_BASE                 0xDC0000U
#define RTC_SIZE                 16U

#define CHIPSTATE_SIZE           65536U
#define CIASTATE_SIZE            4096U

#define ROM_512K_SIZE            0x80000U
#define ROM_1M_SIZE              0x100000U

/* -------------------------------------------------------------------------- */
/* Globals                                                                     */
/* -------------------------------------------------------------------------- */

uint8_t *RAM24bit;
int diss = 0;

static uint32_t g_zero_long = 0;

/* -------------------------------------------------------------------------- */
/* Optional chipset register names                                             */
/* -------------------------------------------------------------------------- */

char *regNames[] = {
    "BLTDDAT","DMACONR","VPOSR","VHPOSR","DSKDATR","JOY0DAT","JOY1DAT","CLXDAT",
    "ADKCONR","POT0DAT","POT1DAT","POTGOR","SERDATR","DSKBYTR","INTENAR","INTREQR",
    "DSKPTH","DSKPTL","DSKLEN","DSKDAT","REFPTR","VPOSW","VHPOSW","COPCON",
    "SERDAT","SERPER","POTGO","JOYTEST","STREQU","STRVBL","STRHOR","STRLONG",
    "BLTCON0","BLTCON1","BLTAFWM","BLTALWM","BLTCPTH","BLTCPTL","BLTBPTH","BLTBPTL",
    "BLTAPTH","BLTAPTL","BLTDPTH","BLTDPTL","BLTSIZE","BLTCON0L","BLTSIZV","BLTSIZH",
    "BLTCMOD","BLTBMOD","BLTAMOD","BLTDMOD","RESERVED00","RESERVED01","RESERVED02","RESERVED03",
    "BLTCDAT","BLTBDAT","BLTADAT","RESERVED04","SPRHDAT","RESERVED05","DENISEID","DSKSYNC",
    "COP1LCH","COP1LCL","COP2LCH","COP2LCL","COPJMP1","COPJMP2","COPINS","DIWSTRT",
    "DIWSTOP","DDFSTRT","DDFSTOP","DMACON","CLXCON","INTENA","INTREQ","ADKCON",
    "AUD0LCH","AUD0LCL","AUD0LEN","AUD0PER","AUD0VOL","AUD0DAT","RESERVED06","RESERVED07",
    "AUD1LCH","AUD1LCL","AUD1LEN","AUD1PER","AUD1VOL","AUD1DAT","RESERVED08","RESERVED09",
    "AUD2LCH","AUD2LCL","AUD2LEN","AUD2PER","AUD2VOL","AUD2DAT","RESERVED10","RESERVED11",
    "AUD3LCH","AUD3LCL","AUD3LEN","AUD3PER","AUD3VOL","AUD3DAT","RESERVED12","RESERVED13",
    "BPL1PTH","BPL1PTL","BPL2PTH","BPL2PTL","BPL3PTH","BPL3PTL","BPL4PTH","BPL4PTL",
    "BPL5PTH","BPL5PTL","BPL6PTH","BPL6PTL","BPL7PTH","BPL7PTL","BPL8PTH","BPL8PTL",
    "BPLCON0","BPLCON1","BPLCON2","BPLCON3","BPL1MOD","BPL2MOD","RESERVED14","RESERVED15",
    "BPL1DAT","BPL2DAT","BPL3DAT","BPL4DAT","BPL5DAT","BPL6DAT","BPL7DAT","BPL8DAT",
    "SPR0PTH","SPR0PTL","SPR1PTH","SPR1PTL","SPR2PTH","SPR2PTL","SPR3PTH","SPR3PTL",
    "SPR4PTH","SPR4PTL","SPR5PTH","SPR5PTL","SPR6PTH","SPR6PTL","SPR7PTH","SPR7PTL",
    "SPR0POS","SPR0CTL","SPR0DATA","SPR0DATB","SPR1POS","SPR1CTL","SPR1DATA","SPR1DATB",
    "SPR2POS","SPR2CTL","SPR2DATA","SPR2DATB","SPR3POS","SPR3CTL","SPR3DATA","SPR3DATB",
    "SPR4POS","SPR4CTL","SPR4DATA","SPR4DATB","SPR5POS","SPR5CTL","SPR5DATA","SPR5DATB",
    "SPR6POS","SPR6CTL","SPR6DATA","SPR6DATB","SPR7POS","SPR7CTL","SPR7DATA","SPR7DATB",
    "COLOR00","COLOR01","COLOR02","COLOR03","COLOR04","COLOR05","COLOR06","COLOR07",
    "COLOR08","COLOR09","COLOR10","COLOR11","COLOR12","COLOR13","COLOR14","COLOR15",
    "COLOR16","COLOR17","COLOR18","COLOR19","COLOR20","COLOR21","COLOR22","COLOR23",
    "COLOR24","COLOR25","COLOR26","COLOR27","COLOR28","COLOR29","COLOR30","COLOR31",
    "HTOTAL","HSSTOP","HBSTRT","HBSTOP","VTOTAL","VSSTOP","VBSTRT","VBSTOP",
    "RESERVED16","RESERVED17","RESERVED18","RESERVED19","RESERVED20","RESERVED21","BEAMCON0","HSSTRT",
    "VSSTRT","HCENTER","DIWHIGH","RESERVED22","RESERVED23","RESERVED24","RESERVED25","RESERVED26",
    "RESERVED27","RESERVED28","RESERVED29","RESERVED30","RESERVED31","RESERVED32","RESERVED33","NO-OP"
};

/* -------------------------------------------------------------------------- */
/* Generic memory helpers                                                      */
/* -------------------------------------------------------------------------- */

static inline uint32_t amiga_mask_addr(uint32_t address) {
    return address & AMIGA_ADDR_MASK;
}

static inline uint8_t mem_read_u8(uint32_t address) {
    return RAM24bit[address];
}

static inline uint16_t mem_read_be16(uint32_t address) {
    return ((uint16_t)RAM24bit[address] << 8)
         | ((uint16_t)RAM24bit[address + 1]);
}

static inline uint32_t mem_read_be32(uint32_t address) {
    return ((uint32_t)RAM24bit[address] << 24)
         | ((uint32_t)RAM24bit[address + 1] << 16)
         | ((uint32_t)RAM24bit[address + 2] << 8)
         |  (uint32_t)RAM24bit[address + 3];
}

static inline void mem_write_u8(uint32_t address, uint8_t value) {
    RAM24bit[address] = value;
}

static inline void mem_write_be16(uint32_t address, uint16_t value) {
    RAM24bit[address]     = (uint8_t)(value >> 8);
    RAM24bit[address + 1] = (uint8_t)(value);
}

static inline void mem_write_be32(uint32_t address, uint32_t value) {
    RAM24bit[address]     = (uint8_t)(value >> 24);
    RAM24bit[address + 1] = (uint8_t)(value >> 16);
    RAM24bit[address + 2] = (uint8_t)(value >> 8);
    RAM24bit[address + 3] = (uint8_t)(value);
}

static inline uint32_t mem_read_sized(uint32_t address, enum DataSize size) {
    switch (size) {
        case m68kByte: return mem_read_u8(address);
        case m68kWord: return mem_read_be16(address);
        case m68kLong: return mem_read_be32(address);
    }
    return 0;
}

static inline void mem_write_sized(uint32_t address, enum DataSize size, uint32_t value) {
    switch (size) {
        case m68kByte:
            mem_write_u8(address, (uint8_t)value);
            break;
        case m68kWord:
            mem_write_be16(address, (uint16_t)value);
            break;
        case m68kLong:
            mem_write_be32(address, value);
            break;
    }
}

static void mem_fill_range(uint32_t start, uint32_t end_inclusive, uint8_t value) {
    for (uint32_t a = start; a <= end_inclusive; ++a) {
        RAM24bit[a] = value;
    }
}

static void mem_zero_block(uint8_t *ptr, size_t size) {
    for (size_t i = 0; i < size; ++i) {
        ptr[i] = 0;
    }
}

/* -------------------------------------------------------------------------- */
/* Address mapping helpers                                                     */
/* -------------------------------------------------------------------------- */

static inline uint32_t chip_mirror_to_chipreg(uint32_t address) {
    return CHIPREG_BASE | (address & CHIPREG_ALIAS_MASK);
}

static inline int is_dmacon_alias(uint32_t address) {
    return (address & CHIPREG_ALIAS_MASK) == DMACON_OFFSET;
}

/* -------------------------------------------------------------------------- */
/* ROM loading                                                                 */
/* -------------------------------------------------------------------------- */

static void clear_rom_regions(void) {
    mem_fill_range(EXT_ROM_START, 0xF7FFFFU, 0x00);
    mem_fill_range(AROS_BANK0_START, AROS_BANK1_END, 0x00);
}

static void log_rom_magic(const char *label, uint32_t address) {
    omega_log_hex(label, mem_read_be32(address));
}

static void copy_external_rom_image(const uint8_t *rom, size_t size) {
    if (rom == NULL || size == 0) {
        return;
    }

    if (size >= ROM_1M_SIZE) {
        omega_host_log("Omega: copying dynamic ROM 1MB (ext+main)");

        for (size_t i = 0; i < ROM_512K_SIZE; ++i) {
            RAM24bit[AROS_EXT_START + i] = rom[i];
        }

        for (size_t i = 0; i < ROM_512K_SIZE; ++i) {
            RAM24bit[AROS_MAIN_START + i] = rom[ROM_512K_SIZE + i];
        }

        log_rom_magic("Omega: ext[0xE00000]", AROS_EXT_START);
        log_rom_magic("Omega: main[0xF80000]", AROS_MAIN_START);
        return;
    }

    {
        const size_t copy_size = (size < ROM_512K_SIZE) ? size : ROM_512K_SIZE;

        omega_host_log("Omega: copying dynamic ROM 512KB");

        for (size_t i = 0; i < copy_size; ++i) {
            RAM24bit[AROS_MAIN_START + i] = rom[i];
        }

        log_rom_magic("Omega: main[0xF80000]", AROS_MAIN_START);
    }
}

static void load_builtin_aros_rom(void) {
#if defined(HAVE_AROS)
    omega_host_log("Omega: dynamic ROM unavailable, using built-in AROS");

    for (size_t i = 0; i < sizeof(aros_main); ++i) {
        RAM24bit[AROS_MAIN_START + i] = aros_main[i];
    }

    for (size_t i = 0; i < sizeof(aros_ext); ++i) {
        RAM24bit[AROS_EXT_START + i] = aros_ext[i];
    }

    log_rom_magic("Omega: ext[0xE00000]", AROS_EXT_START);
    log_rom_magic("Omega: main[0xF80000]", AROS_MAIN_START);
#else
#error "Built-in AROS fallback requested, but AROS headers are not available."
#endif
}

static void load_rom_with_fallback(void) {
    const uint8_t *dyn_rom = omega_host_rom_ptr();
    const size_t dyn_size = omega_host_rom_size();

    if (dyn_rom != (const uint8_t *)0 && dyn_size > 0) {
        copy_external_rom_image(dyn_rom, dyn_size);
        return;
    }

    load_builtin_aros_rom();
}

/* -------------------------------------------------------------------------- */
/* Region handlers                                                             */
/* -------------------------------------------------------------------------- */

static uint32_t handle_chip_ram(uint32_t address,
                                enum DataSize size,
                                enum DataDirection direction,
                                uint32_t value) {
    if (direction == m68kWrite) {
        mem_write_sized(address, size, value);
        return 0;
    }

    return mem_read_sized(address, size);
}

static uint32_t handle_zorro2_unmapped(enum DataSize size) {
    (void)size;
    return 0;
}

static uint32_t handle_cia(uint32_t address,
                           enum DataSize size,
                           enum DataDirection direction,
                           uint32_t value) {
    (void)size;

    if (direction == m68kWrite) {
        WriteCIA(address, value);
        return 0;
    }

    return RAM24bit[address];
}

static uint32_t handle_chip_mirror(uint32_t address,
                                   enum DataSize size,
                                   enum DataDirection direction,
                                   uint32_t value) {
    if (direction == m68kRead) {
        return mem_read_sized(chip_mirror_to_chipreg(address), size);
    }

    if (is_dmacon_alias(address)) {
        WriteChipsetWord(address, value);
    }

    return 0;
}

static uint32_t handle_chip_registers(uint32_t address,
                                      enum DataSize size,
                                      enum DataDirection direction,
                                      uint32_t value) {
    if (direction == m68kRead) {
        return mem_read_sized(address, size);
    }

    switch (size) {
        case m68kByte:
            WriteChipsetByte(address, value);
            break;
        case m68kWord:
            WriteChipsetWord(address, value);
            break;
        case m68kLong:
            WriteChipsetLong(address, value);
            break;
    }

    return 0;
}

static uint32_t autoconfig_open_bus_value(enum DataSize size) {
    switch (size) {
        case m68kByte: return 0xFFU;
        case m68kWord: return 0xFFFFU;
        case m68kLong: return 0xFFFFFFFFU;
    }
    return 0xFFFFFFFFU;
}

static uint32_t handle_rom_space(uint32_t address,
                                 enum DataSize size,
                                 enum DataDirection direction) {
    if (address <= EXT_ROM_END) {
        return (direction == m68kRead) ? mem_read_sized(address, size) : 0;
    }

    if (address <= AUTOCONFIG_END) {
        return autoconfig_open_bus_value(size);
    }

    return (direction == m68kRead) ? mem_read_sized(address, size) : 0;
}

/* -------------------------------------------------------------------------- */
/* Main 24-bit dispatcher                                                      */
/* -------------------------------------------------------------------------- */

unsigned int RAM24BitDespatch(uint32_t address,
                              enum DataSize size,
                              enum DataDirection direction,
                              unsigned int value) {
    address = amiga_mask_addr(address);

    if (address <= CHIP_RAM_END) {
        return handle_chip_ram(address, size, direction, value);
    }

    if (address <= ZORRO2_END) {
        return handle_zorro2_unmapped(size);
    }

    if (address <= SLOW_RAM_END) {
        return 0;
    }

    if (address <= CIA_END) {
        return handle_cia(address, size, direction, value);
    }

    if (address <= CHIP_MIRROR_END) {
        return handle_chip_mirror(address, size, direction, value);
    }

    if (address <= CHIPREG_END) {
        return handle_chip_registers(address, size, direction, value);
    }

    return handle_rom_space(address, size, direction);
}

/* -------------------------------------------------------------------------- */
/* Legacy experimental/debug dispatcher                                        */
/* -------------------------------------------------------------------------- */

unsigned int RAM24BitDespatchEXP(uint32_t address,
                                 enum DataSize size,
                                 enum DataDirection direction,
                                 unsigned int value) {
    address = amiga_mask_addr(address);

    if (address == 0) {
        g_zero_long = (value == 0x48454c50U) ? value : 0;
    }

    if (g_zero_long != 0 && address == 0x100U) {
        omega_host_log("Error Code at 0x100");
    }

    if (address <= CHIP_RAM_END) {
        return handle_chip_ram(address, size, direction, value);
    }

    if (address < SLOW_RAM_START) {
        return 0;
    }

    if (address < CIA_START) {
        return 0;
    }

    if (address <= CIA_END) {
        return handle_cia(address, size, direction, value);
    }

    if (address < CHIPREG_START) {
        return handle_chip_mirror(address, size, direction, value);
    }

    if (address < 0xDA0000U) {
        omega_host_log("Also Reserved");
        (void)m68k_get_reg(NULL, M68K_REG_PC);
        diss = 1;
        return 0;
    }

    if (address < 0xDC0000U) {
        omega_host_log("IDE access");
        return 0;
    }

    if (address < 0xDD0000U) {
        omega_host_log("RTC access");
        return 0;
    }

    if (address < 0xDE0000U) {
        omega_host_log("Reserved access");
        return 0;
    }

    if (address < 0xDF0000U) {
        omega_host_log("Mainboard Resources");
        return 0;
    }

    if (address < 0xE00000U) {
        return handle_chip_registers(address, size, direction, value);
    }

    if (address < 0xE80000U) {
        omega_host_log("Reserved!");
        return 0;
    }

    if (address < 0xF00000U) {
        return 0;
    }

    if (address <= KICK_ROM_END) {
        return mem_read_sized(address, size);
    }

    return 0;
}

/* -------------------------------------------------------------------------- */
/* CIA special read behaviour                                                  */
/* -------------------------------------------------------------------------- */

static inline unsigned int cia_read_icr_a(void) {
    uint8_t pending = RAM24bit[CIA_ICR_A_ADDR];

    if (pending & CIAState->AICRMask) {
        pending |= 0x80;
    }

    RAM24bit[CIA_ICR_A_ADDR] = 0;
    WriteChipsetWord(INTREQ_ADDR, INTREQ_CLR_PORTS);
    return pending;
}

static inline unsigned int cia_read_icr_b(void) {
    uint8_t pending = RAM24bit[CIA_ICR_B_ADDR];

    if (pending & CIAState->BICRMask) {
        pending |= 0x80;
    }

    RAM24bit[CIA_ICR_B_ADDR] = 0;
    WriteChipsetWord(INTREQ_ADDR, INTREQ_CLR_EXTER);
    return pending;
}

static inline unsigned int cia_read_tod_a(uint32_t address) {
    if (address == CIA_ATODH) {
        CIAState->ATODLatch = 1;
        return RAM24bit[address];
    }

    if (address == CIA_ATODL) {
        const uint8_t value = RAM24bit[address];
        CIAState->ATODLatch = 0;
        return value;
    }

    return RAM24BitDespatch(address, m68kByte, m68kRead, 0);
}

static inline unsigned int cia_read_tod_b(uint32_t address) {
    if (address == CIA_BTODH) {
        CIAState->BTODLatch = 1;
        return RAM24bit[address];
    }

    if (address == CIA_BTODL) {
        const uint8_t value = RAM24bit[address];
        CIAState->BTODLatch = 0;
        return value;
    }

    return RAM24BitDespatch(address, m68kByte, m68kRead, 0);
}

/* -------------------------------------------------------------------------- */
/* Musashi callbacks                                                           */
/* -------------------------------------------------------------------------- */

unsigned int cpu_read_byte(unsigned int address) {
    address = amiga_mask_addr(address);

    if (address == CIA_ICR_A_ADDR) {
        return cia_read_icr_a();
    }

    if (address == CIA_ICR_B_ADDR) {
        return cia_read_icr_b();
    }

    if (address == CIA_ATODH || address == CIA_ATODL) {
        return cia_read_tod_a(address);
    }

    if (address == CIA_BTODH || address == CIA_BTODL) {
        return cia_read_tod_b(address);
    }

    return RAM24BitDespatch(address, m68kByte, m68kRead, 0);
}

unsigned int cpu_read_word(unsigned int address) {
    return RAM24BitDespatch(address, m68kWord, m68kRead, 0);
}

unsigned int cpu_read_long(unsigned int address) {
    return RAM24BitDespatch(address, m68kLong, m68kRead, 0);
}

void cpu_write_byte(unsigned int address, unsigned int value) {
    (void)RAM24BitDespatch(address, m68kByte, m68kWrite, value);
}

void cpu_write_word(unsigned int address, unsigned int value) {
    (void)RAM24BitDespatch(address, m68kWord, m68kWrite, value);
}

void cpu_write_long(unsigned int address, unsigned int value) {
    (void)RAM24BitDespatch(address, m68kLong, m68kWrite, value);
}

void cpu_set_fc(unsigned int fc) {
    (void)fc;
}

int cpu_irq_ack(int level) {
    probe_emit(EVT_INTR_ACK, (uint32_t)level, M68K_INT_ACK_AUTOVECTOR);
    return M68K_INT_ACK_AUTOVECTOR;
}

unsigned int cpu_read_word_dasm(unsigned int address) {
    return cpu_read_word(address);
}

unsigned int cpu_read_long_dasm(unsigned int address) {
    return cpu_read_long(address);
}

/* -------------------------------------------------------------------------- */
/* CPU reset                                                                   */
/* -------------------------------------------------------------------------- */

static void cpu_clear_core_registers(void) {
    m68k_set_reg(M68K_REG_PC, 4);

    m68k_set_reg(M68K_REG_D0, 0);
    m68k_set_reg(M68K_REG_D1, 0);
    m68k_set_reg(M68K_REG_D2, 0);
    m68k_set_reg(M68K_REG_D3, 0);
    m68k_set_reg(M68K_REG_D4, 0);
    m68k_set_reg(M68K_REG_D5, 0);
    m68k_set_reg(M68K_REG_D6, 0);
    m68k_set_reg(M68K_REG_D7, 0);

    m68k_set_reg(M68K_REG_A0, 0);
    m68k_set_reg(M68K_REG_A1, 0);
    m68k_set_reg(M68K_REG_A2, 0);
    m68k_set_reg(M68K_REG_A3, 0);
    m68k_set_reg(M68K_REG_A4, 0);
    m68k_set_reg(M68K_REG_A5, 0);
    m68k_set_reg(M68K_REG_A6, 0);
    m68k_set_reg(M68K_REG_A7, 0);
}

void cpu_pulse_reset(void) {
    cpu_clear_core_registers();

    /* Vector 0 (SP) intentionally zero here; ROM sets proper initial stack. */
    mem_write_be32(0, 0x00000000U);
    mem_write_be32(4, KICK_ROM_BOOT_PC);

    FloppyReset();
}

/* -------------------------------------------------------------------------- */
/* Memory / state initialisation                                               */
/* -------------------------------------------------------------------------- */

static void init_amiga_address_space(void) {
    mem_zero_block(RAM24bit, sizeof(Omega_t));
}

static void init_external_state_buffers(void) {
    uint8_t *chipstate_buf = (uint8_t *)CHIPSTATE_PHYS_ADDR;
    uint8_t *ciastate_buf  = (uint8_t *)CIASTATE_PHYS_ADDR;

    mem_zero_block(chipstate_buf, CHIPSTATE_SIZE);
    mem_zero_block(ciastate_buf, CIASTATE_SIZE);

    InitChipset(((Omega_t *)RAM24bit)->chipRAM, chipstate_buf);
    InitCIA(chipstate_buf, ciastate_buf);
}

static void init_aros_custom_banks(void) {
    mem_fill_range(AROS_BANK0_START, AROS_BANK1_END, 0x00);
}

static void init_chip_ram_pattern(void) {
#if defined(CHIPSET_ECS)
    for (uint32_t i = 0; i < 0x100000U; ++i) {
        RAM24bit[i] = 0x00;
    }
#else
    for (uint32_t i = 0; i < 0x040000U; ++i) {
        RAM24bit[i] = 0xFF;
    }
    for (uint32_t i = 0x040000U; i < 0x200000U; ++i) {
        RAM24bit[i] = 0x84;
    }
#endif
}

static void init_rtc_stub(void) {
    RAM24bit[RTC_BASE + 0x0] = 0x00;
    RAM24bit[RTC_BASE + 0x1] = 0xFF;
    RAM24bit[RTC_BASE + 0x2] = 0xFF;
    RAM24bit[RTC_BASE + 0x3] = 0x00;
    RAM24bit[RTC_BASE + 0x4] = 0xFF;
    RAM24bit[RTC_BASE + 0x5] = 0xFF;
    RAM24bit[RTC_BASE + 0x6] = 0xFF;
    RAM24bit[RTC_BASE + 0x7] = 0xFF;
    RAM24bit[RTC_BASE + 0x8] = 0xFF;
    RAM24bit[RTC_BASE + 0x9] = 0xFF;
    RAM24bit[RTC_BASE + 0xA] = 0xFF;
    RAM24bit[RTC_BASE + 0xB] = 0xFF;
    RAM24bit[RTC_BASE + 0xC] = 0xFF;
    RAM24bit[RTC_BASE + 0xD] = 0xFF;
    RAM24bit[RTC_BASE + 0xE] = 0xFF;
    RAM24bit[RTC_BASE + 0xF] = 0xFF;
}

/* -------------------------------------------------------------------------- */
/* Public init                                                                 */
/* -------------------------------------------------------------------------- */

Omega_t *InitRAM(int RAM32bitSize) {
    (void)RAM32bitSize;

    RAM24bit = (uint8_t *)OMEGA_PHYS_ADDR;

    init_amiga_address_space();
    init_external_state_buffers();
    clear_rom_regions();
    init_aros_custom_banks();
    init_chip_ram_pattern();
    load_rom_with_fallback();
    cpu_pulse_reset();
    init_rtc_stub();

    return (Omega_t *)RAM24bit;
}

/* -------------------------------------------------------------------------- */
/* Disassembler helper                                                         */
/* -------------------------------------------------------------------------- */

void make_hex(char *buff, unsigned int pc, unsigned int length) {
    char *ptr = buff;

    for (; length > 0; length -= 2) {
        sprintf(ptr, "%04x", cpu_read_word_dasm(pc));
        pc += 2;
        ptr += 4;

        if (length > 2) {
            *ptr++ = ' ';
        }
    }
}