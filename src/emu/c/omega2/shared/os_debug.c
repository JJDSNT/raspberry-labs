// os_debug.c — AmigaOS structure reader for bare-metal debugging
//
// Inspired by vAmiga OSDebugger (Dirk W. Hoffmann, MPL 2.0)
// Ported to C99 / bare-metal: no stdlib, reads directly from RAM24bit.

#include "os_debug.h"
#include "omega_host.h"
#include "Memory.h"

// ── Big-endian RAM accessors ──────────────────────────────────────────────────

static uint8_t ram8(uint32_t a) {
    if (a >= 0x1000000) return 0xFF;
    return RAM24bit[a];
}

static uint16_t ram16(uint32_t a) {
    return ((uint16_t)ram8(a) << 8) | ram8(a + 1);
}

static uint32_t ram32(uint32_t a) {
    return ((uint32_t)ram8(a)     << 24)
         | ((uint32_t)ram8(a + 1) << 16)
         | ((uint32_t)ram8(a + 2) <<  8)
         |  (uint32_t)ram8(a + 3);
}

static int is_ram_ptr(uint32_t a) {
    return (a >= 0x400) && (a < 0xC00000);
}

// ── Minimal serial output helpers ────────────────────────────────────────────

static const char HEX[] = "0123456789ABCDEF";

// Write a null-terminated string to a fixed buffer, return chars written.
// Used to build lines without printf.
static int put_str(char *buf, int pos, const char *s) {
    while (*s) buf[pos++] = *s++;
    return pos;
}

static int put_hex8(char *buf, int pos, uint8_t v) {
    buf[pos++] = HEX[(v >> 4) & 0xF];
    buf[pos++] = HEX[v & 0xF];
    return pos;
}

static int put_hex32(char *buf, int pos, uint32_t v) {
    for (int s = 28; s >= 0; s -= 4) buf[pos++] = HEX[(v >> s) & 0xF];
    return pos;
}

static int put_dec8(char *buf, int pos, int8_t v) {
    if (v < 0) { buf[pos++] = '-'; v = -v; }
    if (v >= 100) buf[pos++] = '0' + v / 100;
    if (v >= 10)  buf[pos++] = '0' + (v / 10) % 10;
    buf[pos++] = '0' + v % 10;
    return pos;
}

// Read a null-terminated Amiga string (up to 31 chars) into buf[32].
static void read_name(uint32_t ptr, char out[32]) {
    out[0] = '\0';
    if (!is_ram_ptr(ptr) && ptr < 0xF80000) return;
    for (int i = 0; i < 31; i++) {
        char c = (char)ram8(ptr + i);
        if (c == 0) break;
        out[i] = c;
        out[i + 1] = '\0';
    }
}

static void log_line(const char *line) {
    omega_host_log(line);
}

// ── AmigaOS offsets (from OSDebuggerTypes.h / exec/execbase.i) ───────────────

// Node (14 bytes):  +0 ln_Succ, +4 ln_Pred, +8 ln_Type, +9 ln_Pri, +10 ln_Name
#define NODE_SUCC(a)    ram32((a) +  0)
#define NODE_TYPE(a)    ram8 ((a) +  8)
#define NODE_NAME(a)    ram32((a) + 10)

// List (14 bytes):  +0 lh_Head, +4 lh_Tail, +8 lh_TailPred, +12 lh_Type
#define LIST_HEAD(a)    ram32((a) + 0)

// Task offsets (from +0 = tc_Node):
#define TASK_STATE(a)   ram8 ((a) + 15)
#define TASK_IDNEST(a)  ((int8_t)ram8((a) + 16))
#define TASK_TDNEST(a)  ((int8_t)ram8((a) + 17))

// Library offsets (from +0 = lib_Node):
#define LIB_VERSION(a)  ram16((a) + 20)
#define LIB_OPENCNT(a)  ram16((a) + 32)

// ExecBase offsets:
#define EXEC_VERSION(a)   ram16((a) + 20)   // LibNode.lib_Version
#define EXEC_IDNEST(a)    ((int8_t)ram8((a) + 294))
#define EXEC_TDNEST(a)    ((int8_t)ram8((a) + 295))
#define EXEC_ATTNFLAGS(a) ram16((a) + 296)
#define EXEC_THISTASK(a)  ram32((a) + 276)
#define EXEC_LIBLIST(a)   ((a) + 378)  // List struct embedded
#define EXEC_TASKREADY(a) ((a) + 406)
#define EXEC_TASKWAIT(a)  ((a) + 420)

// Node types
#define NT_TASK      1
#define NT_LIBRARY   9
#define NT_PROCESS  13

// Task states
static const char *task_state_str(uint8_t s) {
    switch (s) {
        case 1: return "ADDED";
        case 2: return "RUN";
        case 3: return "READY";
        case 4: return "WAIT";
        case 5: return "EXCEPT";
        case 6: return "REMOVED";
        default: return "?";
    }
}

// ── Dump helpers ─────────────────────────────────────────────────────────────

static void dump_task(uint32_t addr, const char *prefix) {
    char name[32]; read_name(NODE_NAME(addr), name);
    uint8_t  state   = TASK_STATE(addr);
    int8_t   idnest  = TASK_IDNEST(addr);
    int8_t   tdnest  = TASK_TDNEST(addr);
    uint8_t  nt      = NODE_TYPE(addr);

    char buf[80];
    int p = 0;
    p = put_str(buf, p, "[OSDBG] ");
    p = put_str(buf, p, prefix);
    p = put_str(buf, p, "@");
    p = put_hex32(buf, p, addr);
    p = put_str(buf, p, " \"");
    p = put_str(buf, p, name);
    p = put_str(buf, p, "\" ");
    p = put_str(buf, p, nt == NT_PROCESS ? "Process" : "Task");
    p = put_str(buf, p, " state=");
    p = put_str(buf, p, task_state_str(state));
    p = put_str(buf, p, " IDN=");
    p = put_dec8(buf, p, idnest);
    p = put_str(buf, p, " TDN=");
    p = put_dec8(buf, p, tdnest);
    buf[p] = '\0';
    log_line(buf);
}

static void dump_library(uint32_t addr) {
    char name[32]; read_name(NODE_NAME(addr), name);
    uint16_t ver = LIB_VERSION(addr);
    uint16_t cnt = LIB_OPENCNT(addr);

    char buf[80];
    int p = 0;
    p = put_str(buf, p, "[OSDBG]   lib @");
    p = put_hex32(buf, p, addr);
    p = put_str(buf, p, " \"");
    p = put_str(buf, p, name);
    p = put_str(buf, p, "\" v");
    p = put_hex8(buf, p, (uint8_t)(ver >> 8));
    buf[p++] = '.';
    p = put_hex8(buf, p, (uint8_t)ver);
    p = put_str(buf, p, " open=");
    buf[p++] = '0' + (cnt % 10);  // simple for small counts
    buf[p] = '\0';
    log_line(buf);
}

static void dump_list(uint32_t list_addr, const char *label, int is_lib) {
    char hdr[48];
    int p = 0;
    p = put_str(hdr, p, "[OSDBG] ");
    p = put_str(hdr, p, label);
    hdr[p] = '\0';
    log_line(hdr);

    uint32_t node = LIST_HEAD(list_addr);
    int count = 0;
    while (is_ram_ptr(node) && count < 32) {
        if (is_lib) dump_library(node);
        else        dump_task(node, "  task ");
        node = NODE_SUCC(node);
        count++;
    }
    if (count == 0) log_line("[OSDBG]   (empty)");
}

// ── Public entry point ────────────────────────────────────────────────────────

void os_debug_dump(void) {
    log_line("[OSDBG] ---- OS state ----");

    // ExecBase pointer lives at Amiga address 4
    uint32_t eb_ptr = ram32(4);
    if (!is_ram_ptr(eb_ptr)) {
        log_line("[OSDBG] ExecBase not set (addr 4 = 0 or invalid)");
        return;
    }

    // Header
    {
        char buf[64];
        int p = 0;
        p = put_str(buf, p, "[OSDBG] ExecBase @");
        p = put_hex32(buf, p, eb_ptr);
        p = put_str(buf, p, " exec v");
        uint16_t ver = EXEC_VERSION(eb_ptr);
        p = put_hex8(buf, p, (uint8_t)(ver >> 8));
        buf[p++] = '.';
        p = put_hex8(buf, p, (uint8_t)ver);
        buf[p] = '\0';
        log_line(buf);
    }

    // IDNestCnt / TDNestCnt — if > 0, interrupts/tasks are disabled
    {
        int8_t idn = EXEC_IDNEST(eb_ptr);
        int8_t tdn = EXEC_TDNEST(eb_ptr);
        uint16_t attn = EXEC_ATTNFLAGS(eb_ptr);
        char buf[64];
        int p = 0;
        p = put_str(buf, p, "[OSDBG] IDNestCnt=");
        p = put_dec8(buf, p, idn);
        p = put_str(buf, p, " TDNestCnt=");
        p = put_dec8(buf, p, tdn);
        p = put_str(buf, p, " AttnFlags=");
        p = put_hex32(buf, p, attn);
        buf[p] = '\0';
        log_line(buf);
    }

    // ThisTask
    {
        uint32_t tt = EXEC_THISTASK(eb_ptr);
        if (is_ram_ptr(tt)) {
            dump_task(tt, "ThisTask ");
        } else {
            log_line("[OSDBG] ThisTask: none");
        }
    }

    // Library list
    dump_list(EXEC_LIBLIST(eb_ptr),    "LibList:", 1);

    // Task lists
    dump_list(EXEC_TASKREADY(eb_ptr),  "TaskReady:", 0);
    dump_list(EXEC_TASKWAIT(eb_ptr),   "TaskWait:", 0);

    log_line("[OSDBG] ---- end ----");
}
