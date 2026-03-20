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

	styleSummary = lipgloss.NewStyle().
			Foreground(lipgloss.Color("110")).
			MarginBottom(1)
)

// ---------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------

type step int

const (
	stepSelectDemo step = iota
	stepSelectScreen
	stepSelectDisplay
)

// ---------------------------------------------------------------------------
// Opções
// ---------------------------------------------------------------------------

var screenOptions = []demo.ScreenOption{
	{Label: "640x480", Width: 640, Height: 480, Depth: 32},
	{Label: "800x600", Width: 800, Height: 600, Depth: 32},
	{Label: "1024x768", Width: 1024, Height: 768, Depth: 32},
}

var displayOptions = []struct {
	Label string
	Mode  demo.DisplayMode
}{
	{Label: "SDL", Mode: demo.DisplaySDL},
	{Label: "GTK", Mode: demo.DisplayGTK},
	{Label: "Sem janela", Mode: demo.DisplayNone},
}

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

type Model struct {
	demos  []demo.Config
	cursor int

	step step

	Selected *demo.Config
	Screen   demo.ScreenOption
	Display  demo.DisplayMode

	quitting bool
}

func NewModel() Model {
	return Model{
		demos:   demo.All,
		step:    stepSelectDemo,
		cursor:  0,
		Display: demo.DisplaySDL,
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
			if m.cursor < m.maxCursor() {
				m.cursor++
			}

		case "esc":
			switch m.step {
			case stepSelectScreen:
				m.step = stepSelectDemo
				m.cursor = m.selectedDemoIndex()

			case stepSelectDisplay:
				m.step = stepSelectScreen
				m.cursor = m.selectedScreenIndex()
			}

		case "enter", " ":
			switch m.step {
			case stepSelectDemo:
				selected := m.demos[m.cursor]
				m.Selected = &selected
				m.step = stepSelectScreen
				m.cursor = 0

			case stepSelectScreen:
				m.Screen = screenOptions[m.cursor]
				m.step = stepSelectDisplay
				m.cursor = 0

			case stepSelectDisplay:
				m.Display = displayOptions[m.cursor].Mode
				return m, tea.Quit
			}
		}
	}

	return m, nil
}

func (m Model) View() string {
	if m.quitting {
		return ""
	}

	switch m.step {
	case stepSelectDemo:
		return m.viewDemo()

	case stepSelectScreen:
		return m.viewScreenOptions()

	case stepSelectDisplay:
		return m.viewDisplayOptions()
	}

	return ""
}

// ---------------------------------------------------------------------------
// Views
// ---------------------------------------------------------------------------

func (m Model) viewDemo() string {
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

	help := styleHelp.Render("↑/↓ navigate   ↵ select   q quit")
	content := fmt.Sprintf("%s\n%s\n%s", title, items, help)

	return styleBorder.Render(content) + "\n"
}

func (m Model) viewScreenOptions() string {
	title := styleTitle.Render("  Selecione a resolução")
	summary := styleSummary.Render("Demo: " + m.selectedDemoName())

	var items string
	for i, opt := range screenOptions {
		cursor := "  "
		label := opt.Label

		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			label = styleSelected.Render(label)
		} else {
			label = styleNormal.Render(label)
		}

		line := fmt.Sprintf("%s%s", cursor, label)

		if i < len(screenOptions)-1 {
			items += line + "\n"
		} else {
			items += line
		}
	}

	help := styleHelp.Render("↑/↓ navigate   ↵ select   esc back   q quit")
	content := fmt.Sprintf("%s\n%s\n%s\n%s", title, summary, items, help)

	return styleBorder.Render(content) + "\n"
}

func (m Model) viewDisplayOptions() string {
	title := styleTitle.Render("  Selecione o display do QEMU")
	summary := styleSummary.Render(
		fmt.Sprintf("Demo: %s   |   Resolução: %s", m.selectedDemoName(), m.Screen.Label),
	)

	var items string
	for i, opt := range displayOptions {
		cursor := "  "
		label := opt.Label

		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			label = styleSelected.Render(label)
		} else {
			label = styleNormal.Render(label)
		}

		line := fmt.Sprintf("%s%s", cursor, label)

		if i < len(displayOptions)-1 {
			items += line + "\n"
		} else {
			items += line
		}
	}

	help := styleHelp.Render("↑/↓ navigate   ↵ launch   esc back   q quit")
	content := fmt.Sprintf("%s\n%s\n%s\n%s", title, summary, items, help)

	return styleBorder.Render(content) + "\n"
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

func (m Model) maxCursor() int {
	switch m.step {
	case stepSelectDemo:
		if len(m.demos) == 0 {
			return 0
		}
		return len(m.demos) - 1

	case stepSelectScreen:
		return len(screenOptions) - 1

	case stepSelectDisplay:
		return len(displayOptions) - 1
	}

	return 0
}

func (m Model) selectedDemoName() string {
	if m.Selected == nil {
		return ""
	}
	return m.Selected.Name
}

func (m Model) selectedDemoIndex() int {
	if m.Selected == nil {
		return 0
	}

	for i, d := range m.demos {
		if d.BootArg == m.Selected.BootArg {
			return i
		}
	}

	return 0
}

func (m Model) selectedScreenIndex() int {
	for i, s := range screenOptions {
		if s.Width == m.Screen.Width &&
			s.Height == m.Screen.Height &&
			s.Depth == m.Screen.Depth &&
			s.Label == m.Screen.Label {
			return i
		}
	}
	return 0
}
