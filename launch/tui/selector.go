package tui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	"github.com/yourname/launch/demo"
)

// ---------------------------------------------------------------------------
// Estilos
// ---------------------------------------------------------------------------

var (
	styleBorder = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("62")).
			Padding(1, 3)

	styleTitle = lipgloss.NewStyle().
			Bold(true).
			Foreground(lipgloss.Color("62")).
			MarginBottom(1)

	styleCursor = lipgloss.NewStyle().
			Foreground(lipgloss.Color("212")).
			Bold(true)

	styleSelected = lipgloss.NewStyle().
			Foreground(lipgloss.Color("255")).
			Bold(true)

	styleNormal = lipgloss.NewStyle().
			Foreground(lipgloss.Color("245"))

	styleDesc = lipgloss.NewStyle().
			Foreground(lipgloss.Color("240")).
			Italic(true)

	styleHelp = lipgloss.NewStyle().
			Foreground(lipgloss.Color("238")).
			MarginTop(1)
)

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

type Model struct {
	demos    []demo.Config
	cursor   int
	Selected *demo.Config // preenchido ao pressionar Enter
	quitting bool
}

func NewModel() Model {
	return Model{
		demos: demo.All,
	}
}

// ---------------------------------------------------------------------------
// Bubble Tea interface
// ---------------------------------------------------------------------------

func (m Model) Init() tea.Cmd {
	return nil
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {

		case "ctrl+c", "q":
			m.quitting = true
			return m, tea.Quit

		case "up", "k":
			if m.cursor > 0 {
				m.cursor--
			}

		case "down", "j":
			if m.cursor < len(m.demos)-1 {
				m.cursor++
			}

		case "enter", " ":
			selected := m.demos[m.cursor]
			m.Selected = &selected
			return m, tea.Quit
		}
	}
	return m, nil
}

func (m Model) View() string {
	if m.quitting {
		return ""
	}

	title := styleTitle.Render("  Bare Metal Demo Launcher")

	var items string
	for i, d := range m.demos {
		cursor := "  "
		var line string

		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			name := styleSelected.Render(d.Name)
			desc := styleDesc.Render("  " + d.Description)
			line = fmt.Sprintf("%s%s\n%s", cursor, name, desc)
		} else {
			line = fmt.Sprintf("%s%s", cursor, styleNormal.Render(d.Name))
		}

		if i < len(m.demos)-1 {
			items += line + "\n"
		} else {
			items += line
		}
	}

	help := styleHelp.Render("↑/↓  navigate   ↵  launch   q  quit")

	content := fmt.Sprintf("%s\n%s\n%s", title, items, help)
	return styleBorder.Render(content) + "\n"
}
