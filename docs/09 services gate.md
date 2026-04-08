# MC68k-64 Architecture Specification  
## Document 09 — Service Gate Protocol  

**Status:** Stable Draft  
**Version:** 1.0  

---

## 1. Overview

The Service Gate Protocol defines a standardized mechanism for communication between the CPU and system services.

A Service Gate represents a logical device that:

- Exposes a memory-mapped control interface (MMIO)
- Uses shared memory for data exchange
- May execute asynchronously on another core or processing unit

Service Gates unify:

- Hardware devices  
- Firmware services  
- Kernel-managed services  

---

## 2. Design Goals

- Unified device and service abstraction  
- Zero-copy communication  
- SMP-aware operation  
- Explicit capability negotiation  
- Deterministic low-latency signaling  
- Compatibility with legacy discovery (Zorro CFG)  

---

## 3. Architectural Model

A Service Gate consists of:

1. **Control Interface (MMIO)**  
2. **Data Interface (Shared Memory)**  
3. **Event Mechanism (Doorbell / Interrupt)**  

---

## 4. Discovery and Enumeration

Service Gates are discovered through the **Zorro CFG Window**.

Each service provides:

- Vendor ID  
- Device ID  
- Class / Subclass  
- Capability flags  
- Required resources  

After configuration:

- The service is assigned a base address  
- Control registers become accessible  
- Shared memory is allocated  

---

## 5. Control Interface (MMIO)

### 5.1 Standard Register Layout

| Offset | Register | Type | Description |
|--------|----------|------|-------------|
| 0x000 | GATE_MAGIC | RO | Service identifier |
| 0x008 | GATE_STATUS | RO | State flags |
| 0x010 | GATE_CONTROL | RW | Control flags |
| 0x020 | DOORBELL | WO | Vectored signal trigger |
| 0x030 | FEATURES_LO | RO | Capability bits (low) |
| 0x034 | FEATURES_HI | RO | Capability bits (high) |
| 0x038 | FEAT_ACK_LO | RW | Accepted features |
| 0x03C | FEAT_ACK_HI | RW | Accepted features |
| 0x040 | FEAT_COMMIT | WO | Commit negotiated features |
| 0x050 | CMDQ_ADDR | RW | Command queue base |
| 0x058 | CMDQ_SIZE | RW | Command queue size |
| 0x060 | EVTQ_ADDR | RW | Event queue base |
| 0x068 | EVTQ_SIZE | RW | Event queue size |

---

### 5.2 Doorbell Semantics

The DOORBELL register carries a **signal vector**.

Bit assignments:

- Bit 0 → Command queue updated  
- Bit 1 → Event queue consumed  
- Bit 2+ → Service-specific signals  

The system must route signals without scanning all services.

---

### 5.3 Status Flags

Examples:

- READY  
- BUSY  
- ERROR  
- INTERRUPT_PENDING  

---

## 6. Data Interface (Shared Memory)

### 6.1 Overview

Data exchange uses shared memory regions.

These regions:

- Are allocated during configuration  
- Are accessible to CPU and service  
- Must obey alignment and endianness rules  

---

### 6.2 Ring Buffer Layout

```
struct RingHeader {
    // Cache line 0 (consumer)
    consumer_head: AtomicU32,
    padding0: [u8; 60],

    // Cache line 1 (producer)
    producer_tail: AtomicU32,
    padding1: [u8; 60],

    // Cache line 2 (shared)
    capacity: u32,
    elem_size: u32,
    flags: u32,
    padding2: [u8; 52],
}
```

---

### 6.3 Cache Coherency Rules

- Producer updates `producer_tail`  
- Consumer updates `consumer_head`  
- Each field resides in a separate cache line  
- Memory barriers must be used when required  

---

### 6.4 Alignment Requirements

- All elements must follow Document 02 alignment rules  
- No implicit padding is allowed  
- Structures must be layout-stable  

---

## 7. Execution Model

### 7.1 Synchronous vs Asynchronous

- MMIO operations are synchronous  
- Service execution is asynchronous  

---

### 7.2 Multi-Core Operation

Services may run on:

- Other CPU cores  
- Dedicated processors  
- Hardware units  

Communication occurs via:

- Shared memory  
- Doorbell signaling  
- Interrupts  

---

## 8. Feature Negotiation

### 8.1 Protocol

1. Read `FEATURES`  
2. Write supported bits to `FEAT_ACK`  
3. Write to `FEAT_COMMIT`  

### 8.2 Requirements

- Services must not enable unsupported features  
- Drivers must not assume features  

---

## 9. Interrupt and Signaling Model

### 9.1 Interrupt Sources

- Command completion  
- Error conditions  
- Service-specific events  

---

### 9.2 Interrupt Coalescing (Optional)

Services may implement:

- Event batching  
- Time-based coalescing  

---

## 10. Memory Semantics

- Shared memory is coherent  
- MMIO is strongly ordered  
- Service operations may execute out-of-order internally  

---

## 11. Service Lifecycle

States:

- DISCOVERED  
- CONFIGURED  
- READY  
- RUNNING  
- ERROR  
- STOPPED  

---

## 12. Integration with System Architecture

Service Gates:

- Appear as devices in memory map  
- Use MMIO for control  
- Use shared memory for data  
- Are indistinguishable from hardware  

---

## 13. Design Principles

1. Zero-copy communication  
2. Cache-aware design  
3. Explicit synchronization  
4. Feature negotiation mandatory  
5. Asynchronous execution model  
6. Deterministic signaling  

---

## 14. Decisions Deferred

| Topic | Status |
|------|--------|
| Exact feature bit definitions | Deferred |
| Security/isolation model | Deferred |
| Multi-queue support | Deferred |
| NUMA considerations | Deferred |
| Service scheduling policies | Deferred |
| Advanced interrupt routing | Deferred |
| Memory ordering extensions | Deferred |

---

*End of document 09-service-gate.md*