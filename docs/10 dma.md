# MC68k-64 Architecture Specification
## Document 07 — Physical Memory Map

**Status:** Draft  
**Version:** 0.1  

---

## 1. Overview

The MC68k-64 physical address space is 64-bit, linear, and big-endian. All resources — RAM, ROM, chipset, expansion boards, and system devices — exist within a single unified address space. There are no separate I/O spaces.

The map is organized around two principles:

- **Low region:** boot, RAM — predictable, stable, well-known addresses
- **High region:** compatibility, devices, system control — fixed bases, expanding upward

---

## 2. Top-Level Memory Map

```
Address                        Region                    Size
─────────────────────────────────────────────────────────────
0x0000_0000_0000_0000          Boot ROM / Reset vectors  64KB minimum
0x0000_0000_0001_0000          RAM (physical window)     64GB
0x0000_000F_FFFF_FFFF          End of RAM window         —
0x0000_0010_0000_0000          Reserved / future MMU     —
  ...
0x0000_00FF_0000_0000          Chipset (compatibility)   16MB
0x0000_00FF_0100_0000          Zorro CFG window          16MB
0x0000_00FF_0200_0000          Zorro board space         512MB
0x0000_00FF_2200_0000          MMIO system               256MB
0x0000_00FF_3200_0000          Reserved                  —
  ...
0x0000_00FF_FFFF_FFFF          End of implemented space  —
```

Addresses above `0x0000_00FF_FFFF_FFFF` are architecturally valid 64-bit addresses but are reserved for future use. Access to unimplemented regions raises an Access Fault.

---

## 3. Boot ROM / Reset Vectors

```
Base:     0x0000_0000_0000_0000
Size:     64KB minimum, 256KB recommended
Type:     ROM — read-only after reset
```

At reset, VBR is initialized to zero. The processor fetches the initial exception vector table from this region.

The boot ROM contains:

- Initial exception vector table (VBR = 0x0000_0000_0000_0000 at reset)
- Reset entry point — first instruction executed after reset
- Minimal bootstrap code sufficient to initialize RAM and load a bootloader
- Hardware identification and capability flags (read-only, implementation-defined)

After the boot sequence, the supervisor kernel relocates VBR to a RAM-based vector table and the boot ROM becomes inert. The boot ROM region is not writable at any privilege level.

**Reset vector layout (at offset 0 from VBR):**

The vector table follows the format defined in document 05 (Exception Model): each entry is a 64-bit handler address at stride 8. The reset entry (vector 0) contains the initial PC. The initial SSP is loaded from a fixed location in the boot ROM at offset defined by the implementation.

---

## 4. RAM

```
Base:     0x0000_0000_0001_0000
Window:   64GB architectural
Type:     RAM — cacheable, coherent
```

The RAM window is a 64GB architectural reservation. Implementations populate as much of this window as the host hardware provides:

| Platform | Physical RAM | Used portion of window |
|----------|-------------|----------------------|
| Pi 3B | 1GB | 0x0000_0000_0001_0000 – 0x0000_0000_4000_FFFF |
| Pi 4 | 4–8GB | 0x0000_0000_0001_0000 – 0x0000_0001_FFFF_FFFF |
| Radxa Orion | up to 32GB | 0x0000_0000_0001_0000 – 0x0000_0007_FFFF_FFFF |

Addresses within the 64GB window but beyond the installed RAM raise an Access Fault.

The RAM window is the primary workspace for the OS, applications, DMA buffers, and page tables (future MMU).

---

## 5. Reserved Region

```
Base:     0x0000_0010_0000_0000
End:      0x0000_00FE_FFFF_FFFF
Type:     Reserved
```

This region is reserved for future architectural use, including:

- Extended RAM windows for implementations beyond 64GB
- MMU-managed address space extensions
- Future bus fabric regions

Access raises an Access Fault in v0.

---

## 6. Compatibility Region

The compatibility region occupies a contiguous range in the upper part of the implemented space. It contains the chipset, Zorro expansion, and system MMIO — all at fixed, architecturally-defined bases.

```
0x0000_00FF_0000_0000  ┌─────────────────────────────┐
                       │  Chipset (16MB)              │
0x0000_00FF_0100_0000  ├─────────────────────────────┤
                       │  Zorro CFG window (16MB)     │
0x0000_00FF_0200_0000  ├─────────────────────────────┤
                       │  Zorro board space (512MB)   │
0x0000_00FF_2200_0000  ├─────────────────────────────┤
                       │  MMIO system (256MB)         │
0x0000_00FF_3200_0000  ├─────────────────────────────┤
                       │  Reserved                    │
0x0000_00FF_FFFF_FFFF  └─────────────────────────────┘
```

### 6.1 Chipset Region

```
Base:     0x0000_00FF_0000_0000
Size:     16MB
Type:     MMIO — non-cacheable, fixed
```

The chipset region hosts the compatibility chipset — the Amiga-inspired audio, video, and DMA coprocessor complex. The region is fixed and non-configurable.

**Internal layout:** The internal register offsets within the chipset region are compatible with the classical Amiga chipset register layout. Software expecting the classical register organization will find registers at the expected relative offsets from the chipset base.

The absolute base address differs from the original Amiga (`$DFF000`), but the relative layout is preserved. The HAL is responsible for mapping the chipset base into any legacy address expected by ported software.

The chipset region is divided conceptually into:

| Offset | Block | Description |
|--------|-------|-------------|
| +0x000000 | Custom registers | DMA, audio, video control — classical layout |
| +0x080000 | Extended registers | Modern extensions, not present in classical hardware |
| +0x100000 | Reserved | Future chipset expansion |

Access to the chipset region from user mode is subject to MMU protection when active. In v0 without MMU, access is unrestricted by hardware but governed by OS policy.

### 6.2 Zorro CFG Window

```
Base:     0x0000_00FF_0100_0000
Size:     16MB
Type:     MMIO — non-cacheable, autoconfig protocol
```

The Zorro CFG window is the architectural region where expansion boards are enumerated and configured at boot time. The autoconfig protocol operates within this window.

**Autoconfig sequence:**

1. At boot, the firmware/HAL probes the CFG window
2. Each unconfigured board responds at a defined probe address within the window
3. The firmware reads the board's identity and capability descriptor
4. The firmware assigns the board a base address within the Zorro board space
5. The board acknowledges its assigned address and relocates its MMIO response

After autoconfig completes, the CFG window becomes quiescent. Boards respond only in their assigned board space addresses.

**Zorro as a logical bus:**

The Zorro bus is an architectural abstraction, not a physical protocol commitment. The physical transport underlying a Zorro board may be PCIe, USB, an internal virtual device, or any other mechanism. The HAL translates between the physical transport and the Zorro autoconfig semantics. The CPU and OS interact only with the logical Zorro model.

### 6.3 Zorro Board Space

```
Base:     0x0000_00FF_0200_0000
Size:     512MB
Type:     MMIO — non-cacheable, board-assigned
```

After autoconfig, each configured board occupies a sub-region of the board space. The sub-region base and size were negotiated during autoconfig.

Board space is semantically distinct from the system MMIO region:

| Zorro board space | System MMIO |
|-------------------|-------------|
| Logical expansion bus | Architectural machine devices |
| Autoconfig assigned | Fixed architectural bases |
| Board-defined semantics | Machine-defined semantics |
| Backed by PCIe / USB / virtual | Intrinsic to the machine |
| AROS sees as expansion hardware | AROS sees as platform hardware |

Large boards (e.g. framebuffer, network, accelerator) may occupy hundreds of megabytes of board space. The 512MB window accommodates several large boards coexisting.

**Coprocessor buffer model:** A coprocessor with large buffer requirements registers its control registers in the system MMIO region (fixed base) and allocates its data buffer space via Zorro autoconfig (in board space). This separates fixed control from variable-size data regions cleanly.

---

## 7. System MMIO Region

```
Base:     0x0000_00FF_2200_0000
Size:     256MB
Type:     MMIO — non-cacheable, fixed
```

The system MMIO region contains the architectural devices intrinsic to the MC68k-64 platform. All bases are fixed and architecturally defined. Each device block has a guaranteed minimum size of 64KB.

### 7.1 Device Map

```
Address                      Device                  Min size
──────────────────────────────────────────────────────────────
0x0000_00FF_2200_0000        Timer                   64KB
0x0000_00FF_2201_0000        IRQ Controller          64KB
0x0000_00FF_2202_0000        DMA Controller (PL330)  64KB
0x0000_00FF_2203_0000        Debug / UART console    64KB
0x0000_00FF_2204_0000        Reserved (system core)  —
  ...
0x0000_00FF_2210_0000        Coprocessor block       —
  ...
0x0000_00FF_2300_0000        SYSTEM_MMIO_EXPANSION   —
  ...
0x0000_00FF_31FF_FFFF        End of system MMIO      —
```

### 7.2 Guaranteed Minimum Size

Each device block has an architecturally-guaranteed minimum register space of 64KB. Implementations may expose additional registers within a larger block, provided the base address and the minimum register layout are preserved.

This guarantee means:

- The OS can map device registers using the fixed base and a known 64KB window
- Larger implementations do not require OS changes
- Future extensions append registers above the minimum, never reorganize within it

### 7.3 Timer

```
Base:     0x0000_00FF_2200_0000
Size:     64KB minimum
```

Provides periodic and one-shot timer capabilities. Generates Timer IRQ (vector 13). Internal register layout to be defined in the device specification.

### 7.4 IRQ Controller

```
Base:     0x0000_00FF_2201_0000
Size:     64KB minimum
```

Manages external interrupt sources and their mapping to the seven IRQ levels (1–7) presented to the CPU. Provides:

- Per-source enable/disable
- Priority assignment
- Pending interrupt status
- IRQ level identification (read by handler to determine CAUSE detail)

### 7.5 DMA Controller (PL330)

```
Base:     0x0000_00FF_2202_0000
Size:     64KB minimum
```

The DMA controller follows the ARM PL330 DMA-330 specification. Provides:

- Multiple independent channels
- Scatter-gather via memory descriptors
- Operates on physical addresses (bypasses MMU)
- Serves chipset, Zorro boards, and system devices

The PL330 specification is publicly available and serves as the normative reference for this block's register layout.

### 7.6 Debug / UART Console

```
Base:     0x0000_00FF_2203_0000
Size:     64KB minimum
```

Provides a minimal debug output channel available from the earliest stages of boot, before any OS services are active. Used by the bootloader, kernel bring-up, and exception handlers for diagnostic output.

### 7.7 Coprocessor Block

```
Base:     0x0000_00FF_2210_0000
Type:     Fixed base, variable size per coprocessor
```

Reserved for internal coprocessors and accelerators intrinsic to the platform (not expansion boards). Examples include:

- FPU control extension registers (if needed beyond FCSR)
- Crypto accelerator
- Signal processing coprocessor
- NPU / ML accelerator (on platforms that provide one, e.g. Radxa Orion RK3588)

Each coprocessor occupies a sub-block with a fixed base within this region. Sub-block bases are defined per platform and documented in the platform BSP.

### 7.8 SYSTEM_MMIO_EXPANSION

```
Base:     0x0000_00FF_2300_0000
End:      0x0000_00FF_31FF_FFFF
Type:     Reserved for future architectural system devices
```

This region is reserved for future system-level devices that become architectural (i.e. intrinsic to the platform, not expansion boards). Access raises an Access Fault in v0.

---

## 8. Memory Attribute Summary

| Region | Cacheable | Writable | User accessible | Notes |
|--------|-----------|----------|-----------------|-------|
| Boot ROM | No | No | No | Read-only at all privilege levels |
| RAM | Yes | Yes | Yes (with MMU) | Normal memory |
| Reserved | — | — | — | Access Fault |
| Chipset | No | Yes | Supervisor only (MMU) | Side-effect on access |
| Zorro CFG | No | Yes | Supervisor only | Autoconfig protocol |
| Zorro boards | No | Yes | Board-defined | Side-effect on access |
| System MMIO | No | Yes | Supervisor only | Side-effect on access |

In v0, without an active MMU, user-mode access restrictions are advisory. Hardware enforcement requires MMU (document 08).

---

## 9. Decisions Deferred

| Topic | Status | Document |
|-------|--------|----------|
| Boot ROM internal layout and reset SSP location | Deferred | BSP / firmware spec |
| Chipset extended register layout | Deferred | Chipset spec |
| Zorro autoconfig protocol detail | Deferred | Zorro spec |
| IRQ controller register layout | Deferred | Device spec |
| Timer register layout | Deferred | Device spec |
| DMA channel assignment and descriptor format | Deferred | PL330 spec (external) |
| Coprocessor sub-block base assignments | Deferred | Platform BSP |
| MMU-enforced memory protection | Deferred | 08-future.md |

---

14. Implementation Notes (Illustrative)

This section provides non-normative implementation guidance.
It does not define architectural behavior.

14.1 Address Types
typedef uint64_t vaddr_t;   // virtual (CPU)
typedef uint64_t paddr_t;   // physical (architectural)
typedef uint64_t dma_addr_t; // device-visible address
typedef uintptr_t host_addr_t; // host physical / virtual
14.2 DMA Handle
typedef enum {
    DMA_TO_DEVICE,
    DMA_FROM_DEVICE,
    DMA_BIDIR
} dma_dir_t;

typedef struct {
    paddr_t    arch_pa;     // architectural physical address
    uint64_t   len;
    dma_dir_t  dir;

    dma_addr_t dev_addr;    // address given to device

    void      *backend;     // hw or sw backend state
} dma_handle_t;
14.3 DMA API (Minimal)
bool dma_map(device_t *dev,
             paddr_t arch_pa,
             uint64_t len,
             dma_dir_t dir,
             dma_handle_t *out);

void dma_unmap(device_t *dev, dma_handle_t *h);

void dma_sync_for_device(device_t *dev, dma_handle_t *h);
void dma_sync_for_cpu(device_t *dev, dma_handle_t *h);
14.4 Backend Selection
typedef enum {
    DMA_BACKEND_HW,
    DMA_BACKEND_SW
} dma_backend_kind_t;

typedef struct {
    dma_backend_kind_t kind;
    bool need_byteswap;
    bool coherent;
} dma_backend_caps_t;
dma_backend_kind_t dma_select_backend(void)
{
    if (platform_has_iommu())
        return DMA_BACKEND_HW;

    return DMA_BACKEND_SW;
}
14.5 Hardware Backend (SMMU / IOMMU)
bool dma_map_hw(device_t *dev,
                paddr_t arch_pa,
                uint64_t len,
                dma_dir_t dir,
                dma_handle_t *out)
{
    host_addr_t host = arch_to_host(arch_pa, len);

    dma_addr_t iova = iommu_alloc(dev, len);

    iommu_map(dev, iova, host, len, dir);

    out->arch_pa = arch_pa;
    out->len = len;
    out->dir = dir;
    out->dev_addr = iova;
    out->backend = (void*)iova;

    return true;
}
14.6 Software Backend (Fallback)
bool dma_map_sw(device_t *dev,
                paddr_t arch_pa,
                uint64_t len,
                dma_dir_t dir,
                dma_handle_t *out)
{
    void *bounce = alloc_dma_buffer(len);

    if (dir != DMA_FROM_DEVICE)
        arch_copy_to_host(arch_pa, bounce, len);

    out->arch_pa = arch_pa;
    out->len = len;
    out->dir = dir;
    out->dev_addr = host_to_dma_addr(bounce);
    out->backend = bounce;

    return true;
}
14.7 DMA Completion (Software Path)
void dma_sync_for_cpu_sw(device_t *dev, dma_handle_t *h)
{
    if (h->dir != DMA_TO_DEVICE) {
        void *bounce = h->backend;
        host_copy_to_arch(bounce, h->arch_pa, h->len);
    }
}
14.8 Minimal DMA Remap (Window-Based)
typedef struct {
    dma_addr_t base;
    paddr_t    arch_base;
    uint64_t   size;
} dma_window_t;
bool dma_translate(device_t *dev,
                   dma_addr_t da,
                   paddr_t *out)
{
    for (int i = 0; i < dev->nwin; i++) {
        dma_window_t *w = &dev->win[i];

        if (da >= w->base && da < (w->base + w->size)) {
            *out = w->arch_base + (da - w->base);
            return true;
        }
    }
    return false;
}
14.9 Example: Zorro-like Device Using DMA
void device_start_dma(device_t *dev,
                      paddr_t addr,
                      uint64_t len)
{
    dma_handle_t h;

    dma_map(dev, addr, len, DMA_TO_DEVICE, &h);

    mmio_write(dev, REG_DMA_ADDR, h.dev_addr);
    mmio_write(dev, REG_DMA_LEN, len);
    mmio_write(dev, REG_CTRL, START);
}
14.10 Coherence Entry Point

All DMA accesses ultimately resolve through:

uint64_t coh_read(paddr_t addr, int size);
void     coh_write(paddr_t addr, int size, uint64_t val);

This guarantees:

ordering
visibility
uniform access semantics

*End of document 07-memory-map.md*