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

	result, ok := m.(tui.Model)
	if !ok || result.Selected == nil {
		return
	}

	if err := result.Selected.LaunchWithOptions(result.Screen, result.Display); err != nil {
		fmt.Fprintf(os.Stderr, "launch error: %v\n", err)
		os.Exit(1)
	}
}
