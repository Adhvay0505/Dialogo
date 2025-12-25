package ui

import (
	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/canvas"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/dialog"
	"fyne.io/fyne/v2/widget"
	"image/color"
)

// MainUI holds the main application UI state
type MainUI struct {
	window      *fyne.Window
	toggleTheme func()
	isDarkMode  bool
	font        fyne.Resource
	toggle      *toggleSwitch
}

// NewMainUI creates a new MainUI instance
func NewMainUI(window *fyne.Window, toggleTheme func(), isDark bool, font fyne.Resource) *MainUI {
	return &MainUI{
		window:     window, 
		toggleTheme: toggleTheme,
		isDarkMode:  isDark,
		font:        font,
	}
}

// BuildUI constructs the complete main UI
func (m *MainUI) BuildUI() fyne.CanvasObject {
	// Title label
	titleLabel := widget.NewLabel("Dialogo - XMPP Client")
	titleLabel.TextStyle = fyne.TextStyle{Bold: true}
	titleLabel.Alignment = fyne.TextAlignLeading
	
	// FIXED toggle switch
	m.toggle = newToggleSwitch(m.isDarkMode, m.toggleTheme)
	
	// TRUE TOP-RIGHT corner
	header := container.NewBorder(
		nil, nil, nil, m.toggle, titleLabel,
	)
	
	// Login form
	loginLabel := widget.NewLabel("XMPP Login")
	loginLabel.TextStyle = fyne.TextStyle{Bold: true}
	
	jidEntry := widget.NewEntry()
	jidEntry.SetPlaceHolder("user@jabber.example.com")
	
	passwordEntry := widget.NewPasswordEntry()
	passwordEntry.SetPlaceHolder("Password")
	
	connectBtn := widget.NewButton("Connect", func() {
		m.handleConnect(jidEntry.Text, passwordEntry.Text)
	})
	connectBtn.Importance = widget.HighImportance
	
	form := container.NewVBox(
		loginLabel,
		widget.NewForm(
			widget.NewFormItem("JID", jidEntry),
			widget.NewFormItem("Password", passwordEntry),
		),
		connectBtn,
		container.NewPadded(widget.NewLabel("")),
	)

	// Status + Chat
	statusLabel := widget.NewLabel("Status: Disconnected")
	
	chatArea := widget.NewRichTextFromMarkdown("**Chat messages will appear here...**")
	chatArea.Wrapping = fyne.TextWrapWord
	
	messageEntry := widget.NewEntry()
	messageEntry.SetPlaceHolder("Type your message here...")
	
	sendBtn := widget.NewButton("Send", func() {
		m.handleSend(statusLabel, messageEntry.Text)
		messageEntry.SetText("")
	})
	sendBtn.Importance = widget.HighImportance
	
	messageContainer := container.NewMax(messageEntry)
	messageContainer.Resize(fyne.NewSize(900, 60))
	sendContainer := container.NewMax(sendBtn)
	sendContainer.Resize(fyne.NewSize(120, 60))
	
	inputRow := container.NewHBox(messageContainer, sendContainer)

	chatSection := container.NewBorder(nil, inputRow, nil, nil, container.NewScroll(chatArea))

	vsplit := container.NewHSplit(form, chatSection)
	vsplit.SetOffset(0.3)

	content := container.NewBorder(header, statusLabel, nil, nil, vsplit)

	accent := canvas.NewRectangle(color.NRGBA{R: 79, G: 70, B: 229, A: 30})

	return container.NewStack(content, accent)
}

func (m *MainUI) handleConnect(jid, password string) {
	dialog.ShowInformation("Connect", "Connecting to "+jid+"...", *m.window)
}

func (m *MainUI) handleSend(status *widget.Label, message string) {
	if message != "" {
		status.SetText("Status: Message sent!")
	}
}

// FIXED: Proper canvas toggle widget
type toggleSwitch struct {
	widget.BaseWidget
	isDark   bool
	toggleFn func()
	trackBG  *canvas.Circle
	knob     *canvas.Circle
}

func newToggleSwitch(isDark bool, toggleFn func()) *toggleSwitch {
	t := &toggleSwitch{
		isDark:   isDark,
		toggleFn: toggleFn,
	}
	t.ExtendBaseWidget(t)
	
	t.trackBG = canvas.NewCircle(color.NRGBA{R: 60, G: 60, B: 60, A: 255})
	t.trackBG.Resize(fyne.NewSize(66, 36))
	
	t.knob = canvas.NewCircle(color.NRGBA{R: 255, G: 255, B: 255, A: 255})
	t.knob.Resize(fyne.NewSize(28, 28))
	
	t.updateVisuals()
	return t
}

func (t *toggleSwitch) CreateRenderer() fyne.WidgetRenderer {
	container := container.NewWithoutLayout(t.trackBG, t.knob)
	return widget.NewSimpleRenderer(container)
}

func (t *toggleSwitch) MinSize() fyne.Size {
	return fyne.NewSize(75, 45)
}

func (t *toggleSwitch) updateVisuals() {
	if t.isDark {
		t.trackBG.FillColor = color.NRGBA{R: 45, G: 45, B: 45, A: 255}
		t.knob.FillColor = color.NRGBA{R: 255, G: 255, B: 255, A: 255}
		t.knob.Move(fyne.NewPos(34, 4))
	} else {
		t.trackBG.FillColor = color.NRGBA{R: 220, G: 220, B: 220, A: 255}
		t.knob.FillColor = color.NRGBA{R: 30, G: 30, B: 30, A: 255}
		t.knob.Move(fyne.NewPos(4, 4))
	}
}

func (t *toggleSwitch) Tapped(_ *fyne.PointEvent) {
	t.isDark = !t.isDark
	t.updateVisuals()
	t.Refresh()
	if t.toggleFn != nil {
		t.toggleFn()
	}
}
