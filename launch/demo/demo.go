package demo

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

type Config struct {
	Name        string
	Description string
	BootArg     string
	RomFile     string // opcional: ROM default definida em demos.txt
}

type ScreenOption struct {
	Label  string
	Width  int
	Height int
	Depth  int
}

type DisplayMode string

const (
	DisplaySDL  DisplayMode = "sdl"
	DisplayGTK  DisplayMode = "gtk"
	DisplayNone DisplayMode = "none"
)

type LaunchSelection struct {
	ROMFile  string
	DiskFile string // df0
	HDFFile  string // hd0 (opcional)
}

// ---------------------------------------------------------------------------
// Dados
// ---------------------------------------------------------------------------

var All []Config

// ---------------------------------------------------------------------------
// Constantes
// ---------------------------------------------------------------------------

const (
	dtbBaseName    = "bcm2710-rpi-3-b-plus.dtb"
	dtbPatchedName = "bcm2710-rpi-3-b-plus-patched.dtb"
)

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

func init() {
	path := demosFilePath()

	configs, err := loadConfigsFromFile(path)
	if err != nil {
		fmt.Fprintf(os.Stderr, "erro carregando demos (%s): %v\n", path, err)
		os.Exit(1)
	}

	if len(configs) == 0 {
		fmt.Fprintf(os.Stderr, "nenhum demo encontrado em %s\n", path)
		os.Exit(1)
	}

	All = configs
}

func demosFilePath() string {
	if p := os.Getenv("DEMOS_FILE"); p != "" {
		return p
	}
	return "demo/demos.txt"
}

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

func dtbDir() string {
	if d := os.Getenv("DTB_DIR"); d != "" {
		return d
	}
	return "dtb_qemu"
}

func kernelPath() string {
	if p := os.Getenv("KERNEL_PATH"); p != "" {
		return p
	}
	return "kernel8.img"
}

func sdImgPath() string {
	if p := os.Getenv("SD_IMG_PATH"); p != "" {
		return p
	}
	return ""
}

func disksDir() string {
	if p := os.Getenv("DISKS_DIR"); p != "" {
		return p
	}
	return "../disks"
}

// ---------------------------------------------------------------------------
// Listagem de arquivos em disks/
// ---------------------------------------------------------------------------

func listFilesByExt(dir string, exts ...string) ([]string, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}

	allowed := map[string]bool{}
	for _, ext := range exts {
		allowed[strings.ToLower(ext)] = true
	}

	var out []string
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		ext := strings.ToLower(filepath.Ext(e.Name()))
		if allowed[ext] {
			out = append(out, e.Name())
		}
	}

	sort.Strings(out)
	return out, nil
}

func AvailableROMs() ([]string, error) {
	return listFilesByExt(disksDir(), ".rom", ".bin")
}

func AvailableDisks() ([]string, error) {
	return listFilesByExt(disksDir(), ".adf")
}

func AvailableHDFs() ([]string, error) {
	return listFilesByExt(disksDir(), ".hdf")
}

// ---------------------------------------------------------------------------
// Launch
// ---------------------------------------------------------------------------

func (c *Config) LaunchWithOptions(screen ScreenOption, display DisplayMode, sel LaunchSelection) error {
	bootargs := fmt.Sprintf(
		"demo=%s width=%d height=%d depth=%d",
		c.BootArg, screen.Width, screen.Height, screen.Depth,
	)

	if c.BootArg == "omega" {
		if sel.DiskFile != "" {
			bootargs += " df0=" + sel.DiskFile
		}
		if sel.HDFFile != "" {
			bootargs += " hd0=" + sel.HDFFile
		}

		rom := sel.ROMFile
		if rom == "" {
			rom = c.RomFile
		}
		if rom != "" {
			bootargs += " rom=" + rom
		}
	}

	dir := dtbDir()
	base := filepath.Join(dir, dtbBaseName)
	patched := filepath.Join(dir, dtbPatchedName)

	if err := patchDTB(base, patched, bootargs); err != nil {
		return fmt.Errorf("DTB patch: %w", err)
	}

	return runQEMU(kernelPath(), patched, display, screen, sdImgPath())
}

// ---------------------------------------------------------------------------
// TXT loader
// ---------------------------------------------------------------------------

func loadConfigsFromFile(path string) ([]Config, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("não foi possível abrir arquivo: %w", err)
	}
	defer f.Close()

	var configs []Config
	scanner := bufio.NewScanner(f)

	lineNum := 0
	for scanner.Scan() {
		lineNum++
		line := strings.TrimSpace(scanner.Text())

		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		parts := strings.Split(line, "|")
		if len(parts) < 3 || len(parts) > 4 {
			return nil, fmt.Errorf("linha %d inválida (formato: Nome|Descrição|bootarg[|rom])", lineNum)
		}

		cfg := Config{
			Name:        strings.TrimSpace(parts[0]),
			Description: strings.TrimSpace(parts[1]),
			BootArg:     strings.TrimSpace(parts[2]),
		}
		if len(parts) == 4 {
			cfg.RomFile = strings.TrimSpace(parts[3])
		}

		if cfg.Name == "" {
			return nil, fmt.Errorf("linha %d inválida: nome vazio", lineNum)
		}
		if cfg.BootArg == "" {
			return nil, fmt.Errorf("linha %d inválida: bootarg vazio", lineNum)
		}

		configs = append(configs, cfg)
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("erro lendo arquivo: %w", err)
	}

	return configs, nil
}

// ---------------------------------------------------------------------------
// DTB patch
// ---------------------------------------------------------------------------

func patchDTB(base, patched, bootargs string) error {
	if _, err := os.Stat(base); err != nil {
		return fmt.Errorf("DTB base não encontrado em %s", base)
	}

	src, err := os.ReadFile(base)
	if err != nil {
		return fmt.Errorf("erro lendo DTB base: %w", err)
	}

	if err := os.WriteFile(patched, src, 0644); err != nil {
		return fmt.Errorf("erro escrevendo DTB patched: %w", err)
	}

	cmd := exec.Command("fdtput", "-ts", patched, "/chosen", "bootargs", bootargs)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		_ = os.Remove(patched)
		return fmt.Errorf("erro executando fdtput: %w", err)
	}

	return nil
}

// ---------------------------------------------------------------------------
// QEMU
// ---------------------------------------------------------------------------

func runQEMU(kernel, dtb string, display DisplayMode, screen ScreenOption, sdImg string) error {
	effectiveDisplay := display
	if display == DisplaySDL && (screen.Width != 640 || screen.Height != 480) {
		fmt.Fprintf(os.Stderr,
			"[WARN] SDL não suporta %dx%d no raspi3b emulado — usando GTK\n",
			screen.Width, screen.Height)
		effectiveDisplay = DisplayGTK
	}

	args := []string{
		"-M", "raspi3b",
		"-cpu", "cortex-a53",
		"-kernel", kernel,
		"-dtb", dtb,
		"-serial", "stdio",
		"-display", string(effectiveDisplay),
	}

	if sdImg != "" {
		if _, err := os.Stat(sdImg); err == nil {
			args = append(args, "-drive", fmt.Sprintf("file=%s,format=raw,if=sd", sdImg))
			fmt.Fprintf(os.Stderr, "[SD] Usando %s\n", sdImg)
		}
	}

	cmd := exec.Command("qemu-system-aarch64", args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}
