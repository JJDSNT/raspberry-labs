package demo

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

type Config struct {
	Name        string
	Description string
	BootArg     string
}

var All = []Config{
	{Name: "Gradient", Description: "Gradiente de cores com retângulos", BootArg: "gradient"},
	{Name: "Test Pattern", Description: "Grade de cores para calibração", BootArg: "testpattern"},
	{Name: "Raster Bars", Description: "Barras de cor sincronizadas com raster", BootArg: "rasterbars"},
	{Name: "Starfield", Description: "Campo de estrelas 3D em perspectiva", BootArg: "starfield"},
	{Name: "Plasma", Description: "Efeito plasma com tabela de senos", BootArg: "plasma"},
	{Name: "Flame", Description: "Simulação de fogo procedural", BootArg: "flame"},
}

const (
	dtbBaseName    = "bcm2710-rpi-3-b-plus.dtb"
	dtbPatchedName = "bcm2710-rpi-3-b-plus-patched.dtb"

	defaultWidth  = 640
	defaultHeight = 480
	defaultDepth  = 32
)

// dtbDir retorna o diretório onde ficam os DTBs.
// DTB_DIR é setado pelo run.sh com path absoluto.
func dtbDir() string {
	if d := os.Getenv("DTB_DIR"); d != "" {
		return d
	}
	return "../dtb"
}

// kernelPath retorna o path absoluto do kernel8.img.
func kernelPath() string {
	if p := os.Getenv("KERNEL_PATH"); p != "" {
		return p
	}
	return "../kernel8.img"
}

func (c *Config) Launch() error {
	bootargs := fmt.Sprintf(
		"demo=%s width=%d height=%d depth=%d",
		c.BootArg, defaultWidth, defaultHeight, defaultDepth,
	)

	dir := dtbDir()
	base := filepath.Join(dir, dtbBaseName)
	patched := filepath.Join(dir, dtbPatchedName)

	if err := patchDTB(base, patched, bootargs); err != nil {
		return fmt.Errorf("DTB patch: %w", err)
	}

	return runQEMU(kernelPath(), patched)
}

// patchDTB copia o DTB base para patched e injeta os bootargs no /chosen.
func patchDTB(base, patched, bootargs string) error {
	if _, err := os.Stat(base); err != nil {
		return fmt.Errorf(
			"DTB base não encontrado em %s\n"+
				"Execute ./run.sh para baixá-lo automaticamente", base)
	}

	src, err := os.ReadFile(base)
	if err != nil {
		return fmt.Errorf("lendo DTB base: %w", err)
	}
	if err := os.WriteFile(patched, src, 0644); err != nil {
		return fmt.Errorf("escrevendo DTB patched: %w", err)
	}

	patch := exec.Command("fdtput", "-ts", patched, "/chosen", "bootargs", bootargs)
	patch.Stdout = os.Stdout
	patch.Stderr = os.Stderr
	if err := patch.Run(); err != nil {
		_ = os.Remove(patched)
		return fmt.Errorf("fdtput: %w", err)
	}

	return nil
}

func runQEMU(kernel, dtb string) error {
	cmd := exec.Command(
		"qemu-system-aarch64",
		"-M", "raspi3b",
		"-cpu", "cortex-a53",
		"-kernel", kernel,
		"-dtb", dtb,
		"-serial", "stdio",
		"-display", "sdl",
	)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}
