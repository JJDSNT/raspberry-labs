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

	styleNote = lipgloss.NewStyle().
			Foreground(lipgloss.Color("240")).
			Italic(true)
)

// ---------------------------------------------------------------------------
// Steps — display vem antes da resolução
// ---------------------------------------------------------------------------

type step int

const (
	stepSelectDemo step = iota
	stepSelectDisplay
	stepSelectScreen // pulado quando display = SDL
)

// ---------------------------------------------------------------------------
// Opções
// ---------------------------------------------------------------------------

var allScreenOptions = []demo.ScreenOption{
	{Label: "640x480", Width: 640, Height: 480, Depth: 32},
	{Label: "800x600", Width: 800, Height: 600, Depth: 32},
	{Label: "1024x768", Width: 1024, Height: 768, Depth: 32},
}

var sdlScreenOptions = []demo.ScreenOption{
	{Label: "640x480", Width: 640, Height: 480, Depth: 32},
}

// GTK é o padrão — suporta qualquer resolução.
// SDL é limitado a 640x480 no raspi3b emulado.
var displayOptions = []struct {
	Label string
	Note  string
	Mode  demo.DisplayMode
}{
	{Label: "GTK", Note: "qualquer resolução", Mode: demo.DisplayGTK},
	{Label: "SDL", Note: "640x480 apenas", Mode: demo.DisplaySDL},
	{Label: "Sem janela", Note: "headless / CI", Mode: demo.DisplayNone},
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
		Display: demo.DisplayGTK, // padrão
		Screen:  allScreenOptions[0],
	}
}

// ---------------------------------------------------------------------------
// Helpers de opções de tela
// ---------------------------------------------------------------------------

func (m Model) currentScreenOptions() []demo.ScreenOption {
	if m.Display == demo.DisplaySDL {
		return sdlScreenOptions
	}
	return allScreenOptions
}

func (m Model) skipScreenStep() bool {
	// SDL tem só uma resolução — não precisa de step extra
	return m.Display == demo.DisplaySDL || m.Display == demo.DisplayNone
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
			case stepSelectDisplay:
				m.step = stepSelectDemo
				m.cursor = m.selectedDemoIndex()

			case stepSelectScreen:
				m.step = stepSelectDisplay
				m.cursor = m.selectedDisplayIndex()
			}

		case "enter", " ":
			switch m.step {
			case stepSelectDemo:
				selected := m.demos[m.cursor]
				m.Selected = &selected
				m.step = stepSelectDisplay
				m.cursor = m.selectedDisplayIndex()

			case stepSelectDisplay:
				m.Display = displayOptions[m.cursor].Mode

				if m.skipScreenStep() {
					// SDL/headless: resolve a tela automaticamente e lança
					opts := m.currentScreenOptions()
					m.Screen = opts[0]
					return m, tea.Quit
				}

				// GTK: vai para seleção de resolução
				m.step = stepSelectScreen
				m.cursor = m.selectedScreenIndex()

			case stepSelectScreen:
				opts := m.currentScreenOptions()
				m.Screen = opts[m.cursor]
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
	case stepSelectDisplay:
		return m.viewDisplayOptions()
	case stepSelectScreen:
		return m.viewScreenOptions()
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

	help := styleHelp.Render("↑/↓ navegar   ↵ selecionar   q sair")
	content := fmt.Sprintf("%s\n%s\n%s", title, items, help)

	return styleBorder.Render(content) + "\n"
}

func (m Model) viewDisplayOptions() string {
	title := styleTitle.Render("  Selecione o display do QEMU")
	summary := styleSummary.Render("Demo: " + m.selectedDemoName())

	var items string
	for i, opt := range displayOptions {
		cursor := "  "
		label := opt.Label
		note := styleNote.Render("  — " + opt.Note)

		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			label = styleSelected.Render(label)
		} else {
			label = styleNormal.Render(label)
		}

		line := fmt.Sprintf("%s%s%s", cursor, label, note)

		if i < len(displayOptions)-1 {
			items += line + "\n"
		} else {
			items += line
		}
	}

	help := styleHelp.Render("↑/↓ navegar   ↵ selecionar   esc voltar   q sair")
	content := fmt.Sprintf("%s\n%s\n%s\n%s", title, summary, items, help)

	return styleBorder.Render(content) + "\n"
}

func (m Model) viewScreenOptions() string {
	title := styleTitle.Render("  Selecione a resolução")
	summary := styleSummary.Render(
		fmt.Sprintf("Demo: %s   |   Display: %s", m.selectedDemoName(), m.selectedDisplayLabel()),
	)

	opts := m.currentScreenOptions()
	var items string
	for i, opt := range opts {
		cursor := "  "
		label := opt.Label

		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			label = styleSelected.Render(label)
		} else {
			label = styleNormal.Render(label)
		}

		line := fmt.Sprintf("%s%s", cursor, label)

		if i < len(opts)-1 {
			items += line + "\n"
		} else {
			items += line
		}
	}

	help := styleHelp.Render("↑/↓ navegar   ↵ lançar   esc voltar   q sair")
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

	case stepSelectDisplay:
		return len(displayOptions) - 1

	case stepSelectScreen:
		return len(m.currentScreenOptions()) - 1
	}

	return 0
}

func (m Model) selectedDemoName() string {
	if m.Selected == nil {
		return ""
	}
	return m.Selected.Name
}

func (m Model) selectedDisplayLabel() string {
	for _, opt := range displayOptions {
		if opt.Mode == m.Display {
			return opt.Label
		}
	}
	return ""
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

func (m Model) selectedDisplayIndex() int {
	for i, opt := range displayOptions {
		if opt.Mode == m.Display {
			return i
		}
	}
	return 0
}

func (m Model) selectedScreenIndex() int {
	opts := m.currentScreenOptions()
	for i, s := range opts {
		if s.Width == m.Screen.Width &&
			s.Height == m.Screen.Height &&
			s.Depth == m.Screen.Depth {
			return i
		}
	}
	return 0
}
