package tui

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	"github.com/yourname/launch/demo"
)

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

const (
	noRomOption  = "(sem rom)"
	noDiskOption = "(sem disco)"
	noHDFOption  = "(sem hdf)"
)

type step int

const (
	stepSelectDemo step = iota
	stepSelectROM
	stepSelectDisk
	stepSelectHDF
	stepSelectDisplay
	stepSelectScreen
)

var allScreenOptions = []demo.ScreenOption{
	{Label: "640x480", Width: 640, Height: 480, Depth: 32},
	{Label: "800x600", Width: 800, Height: 600, Depth: 32},
	{Label: "1024x768", Width: 1024, Height: 768, Depth: 32},
}

var sdlScreenOptions = []demo.ScreenOption{
	{Label: "640x480", Width: 640, Height: 480, Depth: 32},
}

var displayOptions = []struct {
	Label string
	Note  string
	Mode  demo.DisplayMode
}{
	{Label: "GTK", Note: "qualquer resolução", Mode: demo.DisplayGTK},
	{Label: "SDL", Note: "640x480 apenas", Mode: demo.DisplaySDL},
	{Label: "Sem janela", Note: "headless / CI", Mode: demo.DisplayNone},
}

type Model struct {
	demos  []demo.Config
	cursor int
	step   step

	Selected *demo.Config
	Screen   demo.ScreenOption
	Display  demo.DisplayMode

	roms        []string
	disks       []string
	hdfs        []string
	SelectedROM  string
	SelectedDisk string
	SelectedHDF  string

	quitting bool
	err      error
}

func NewModel() Model {
	roms, _ := demo.AvailableROMs()
	disks, _ := demo.AvailableDisks()
	hdfs, _ := demo.AvailableHDFs()

	roms = append([]string{noRomOption}, roms...)
	disks = append([]string{noDiskOption}, disks...)
	hdfs = append([]string{noHDFOption}, hdfs...)

	return Model{
		demos:   demo.All,
		step:    stepSelectDemo,
		cursor:  0,
		Display: demo.DisplayGTK,
		Screen:  allScreenOptions[0],
		roms:    roms,
		disks:   disks,
		hdfs:    hdfs,
	}
}

func (m Model) currentScreenOptions() []demo.ScreenOption {
	if m.Display == demo.DisplaySDL {
		return sdlScreenOptions
	}
	return allScreenOptions
}

func (m Model) skipScreenStep() bool {
	return m.Display == demo.DisplaySDL || m.Display == demo.DisplayNone
}

func (m Model) needsMediaSelection() bool {
	return m.Selected != nil && m.Selected.BootArg == "omega"
}

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
			case stepSelectROM:
				m.step = stepSelectDemo
				m.cursor = m.selectedDemoIndex()
			case stepSelectDisk:
				m.step = stepSelectROM
				m.cursor = m.selectedROMIndex()
			case stepSelectHDF:
				m.step = stepSelectDisk
				m.cursor = m.selectedDiskIndex()
			case stepSelectDisplay:
				if m.needsMediaSelection() {
					m.step = stepSelectHDF
					m.cursor = m.selectedHDFIndex()
				} else {
					m.step = stepSelectDemo
					m.cursor = m.selectedDemoIndex()
				}
			case stepSelectScreen:
				m.step = stepSelectDisplay
				m.cursor = m.selectedDisplayIndex()
			}

		case "enter", " ":
			switch m.step {
			case stepSelectDemo:
				selected := m.demos[m.cursor]
				m.Selected = &selected

				if m.needsMediaSelection() {
					m.step = stepSelectROM
					m.cursor = m.selectedROMIndex()
				} else {
					m.step = stepSelectDisplay
					m.cursor = m.selectedDisplayIndex()
				}

			case stepSelectROM:
				if len(m.roms) > 0 {
					chosen := m.roms[m.cursor]
					if chosen == noRomOption {
						m.SelectedROM = ""
					} else {
						m.SelectedROM = chosen
					}
				}
				m.step = stepSelectDisk
				m.cursor = m.selectedDiskIndex()

			case stepSelectDisk:
				if len(m.disks) > 0 {
					chosen := m.disks[m.cursor]
					if chosen == noDiskOption {
						m.SelectedDisk = ""
					} else {
						m.SelectedDisk = chosen
					}
				}
				m.step = stepSelectHDF
				m.cursor = m.selectedHDFIndex()

			case stepSelectHDF:
				if len(m.hdfs) > 0 {
					chosen := m.hdfs[m.cursor]
					if chosen == noHDFOption {
						m.SelectedHDF = ""
					} else {
						m.SelectedHDF = chosen
					}
				}
				m.step = stepSelectDisplay
				m.cursor = m.selectedDisplayIndex()

			case stepSelectDisplay:
				m.Display = displayOptions[m.cursor].Mode

				if m.skipScreenStep() {
					opts := m.currentScreenOptions()
					m.Screen = opts[0]
					return m, tea.Quit
				}

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
	case stepSelectROM:
		return m.viewROMs()
	case stepSelectDisk:
		return m.viewDisks()
	case stepSelectHDF:
		return m.viewHDFs()
	case stepSelectDisplay:
		return m.viewDisplayOptions()
	case stepSelectScreen:
		return m.viewScreenOptions()
	}

	return ""
}

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

func (m Model) viewROMs() string {
	title := styleTitle.Render("  Selecione a ROM")

	romLabel := m.SelectedROM
	if romLabel == "" {
		romLabel = "nenhuma"
	}

	summary := styleSummary.Render(fmt.Sprintf(
		"Demo: %s   |   ROM: %s",
		m.selectedDemoName(),
		romLabel,
	))

	return styleBorder.Render(fmt.Sprintf("%s\n%s\n%s\n%s",
		title,
		summary,
		m.renderSimpleList(m.roms),
		styleHelp.Render("↑/↓ navegar   ↵ selecionar   esc voltar   q sair"),
	)) + "\n"
}

func (m Model) viewDisks() string {
	title := styleTitle.Render("  Selecione o disco (DF0)")

	romLabel := m.SelectedROM
	if romLabel == "" {
		romLabel = "nenhuma"
	}

	summary := styleSummary.Render(fmt.Sprintf(
		"Demo: %s   |   ROM: %s",
		m.selectedDemoName(),
		romLabel,
	))

	return styleBorder.Render(fmt.Sprintf("%s\n%s\n%s\n%s",
		title,
		summary,
		m.renderSimpleList(m.disks),
		styleHelp.Render("↑/↓ navegar   ↵ selecionar   esc voltar   q sair"),
	)) + "\n"
}

func (m Model) viewHDFs() string {
	title := styleTitle.Render("  Selecione o disco rígido (HD0)")

	romLabel := m.SelectedROM
	if romLabel == "" {
		romLabel = "nenhuma"
	}

	diskLabel := m.SelectedDisk
	if diskLabel == "" {
		diskLabel = "nenhum"
	}

	summary := styleSummary.Render(fmt.Sprintf(
		"Demo: %s   |   ROM: %s   |   DF0: %s",
		m.selectedDemoName(),
		romLabel,
		diskLabel,
	))

	return styleBorder.Render(fmt.Sprintf("%s\n%s\n%s\n%s",
		title,
		summary,
		m.renderSimpleList(m.hdfs),
		styleHelp.Render("↑/↓ navegar   ↵ selecionar   esc voltar   q sair"),
	)) + "\n"
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

func (m Model) renderSimpleList(items []string) string {
	if len(items) == 0 {
		return styleNote.Render("  nenhum arquivo encontrado em ../disks")
	}

	out := ""
	for i, item := range items {
		cursor := "  "
		label := styleNormal.Render(item)
		if i == m.cursor {
			cursor = styleCursor.Render("▶ ")
			label = styleSelected.Render(item)
		}

		line := fmt.Sprintf("%s%s", cursor, label)
		if i < len(items)-1 {
			out += line + "\n"
		} else {
			out += line
		}
	}
	return out
}

func (m Model) maxCursor() int {
	switch m.step {
	case stepSelectDemo:
		if len(m.demos) == 0 {
			return 0
		}
		return len(m.demos) - 1
	case stepSelectROM:
		if len(m.roms) == 0 {
			return 0
		}
		return len(m.roms) - 1
	case stepSelectDisk:
		if len(m.disks) == 0 {
			return 0
		}
		return len(m.disks) - 1
	case stepSelectHDF:
		if len(m.hdfs) == 0 {
			return 0
		}
		return len(m.hdfs) - 1
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
		if s.Width == m.Screen.Width && s.Height == m.Screen.Height && s.Depth == m.Screen.Depth {
			return i
		}
	}
	return 0
}

func (m Model) selectedROMIndex() int {
	if m.SelectedROM == "" {
		for i, rom := range m.roms {
			if rom == noRomOption {
				return i
			}
		}
		return 0
	}

	for i, rom := range m.roms {
		if rom == m.SelectedROM {
			return i
		}
	}
	return 0
}

func (m Model) IsQuitting() bool {
	return m.quitting
}

func (m Model) selectedDiskIndex() int {
	if m.SelectedDisk == "" {
		for i, disk := range m.disks {
			if disk == noDiskOption {
				return i
			}
		}
		return 0
	}

	for i, disk := range m.disks {
		if disk == m.SelectedDisk {
			return i
		}
	}
	return 0
}

func (m Model) selectedHDFIndex() int {
	if m.SelectedHDF == "" {
		for i, h := range m.hdfs {
			if h == noHDFOption {
				return i
			}
		}
		return 0
	}

	for i, h := range m.hdfs {
		if h == m.SelectedHDF {
			return i
		}
	}
	return 0
}
