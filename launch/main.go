package main

import (
	"fmt"
	"os"

	tea "github.com/charmbracelet/bubbletea"

	"github.com/yourname/launch/tui"
)

func main() {
	p := tea.NewProgram(tui.NewModel(), tea.WithAltScreen())
	m, err := p.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	// Após o TUI fechar, executa o demo selecionado
	if result, ok := m.(tui.Model); ok && result.Selected != nil {
		if err := result.Selected.Launch(); err != nil {
			fmt.Fprintf(os.Stderr, "launch error: %v\n", err)
			os.Exit(1)
		}
	}
}
