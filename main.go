package main

import (
	"dialogo/internal/ui"
	"fyne.io/fyne/v2/app"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2"
	"image/color"
)

func main() {
	a := app.NewWithID("com.example.dialogo")
	
	// Track dark mode state
	isDarkMode := true
	a.Settings().SetTheme(&darkTheme{})
	
	window := a.NewWindow("Dialogo")
	window.Resize(fyne.NewSize(1200, 800))
	window.CenterOnScreen()
	
	// Pass toggle function + dark mode state to UI (no font for now)
	toggleTheme := func() {
		isDarkMode = !isDarkMode
		if isDarkMode {
			a.Settings().SetTheme(&darkTheme{})
		} else {
			a.Settings().SetTheme(theme.LightTheme())
		}
		window.Content().Refresh()
	}
	
	mainUI := ui.NewMainUI(&window, toggleTheme, isDarkMode, nil)
	window.SetContent(mainUI.BuildUI())
	
	window.ShowAndRun()
}

type darkTheme struct{}

func (c *darkTheme) Color(name fyne.ThemeColorName, variant fyne.ThemeVariant) color.Color {
	switch name {
	case theme.ColorNameBackground:
		return color.NRGBA{R: 30, G: 30, B: 30, A: 255}
	case theme.ColorNameForeground:
		return color.NRGBA{R: 220, G: 220, B: 220, A: 255}
	case theme.ColorNameButton:
		return color.NRGBA{R: 70, G: 70, B: 70, A: 255}
	case theme.ColorNameInputBackground:
		return color.NRGBA{R: 45, G: 45, B: 45, A: 255}
	case theme.ColorNamePrimary:
		return color.NRGBA{R: 100, G: 100, B: 100, A: 255}
	case theme.ColorNameDisabledButton:
		return color.NRGBA{R: 60, G: 60, B: 60, A: 255}
	case theme.ColorNameDisabled:
		return color.NRGBA{R: 80, G: 80, B: 80, A: 255}
	default:
		return theme.DefaultTheme().Color(name, variant)
	}
}

func (c *darkTheme) Font(textStyle fyne.TextStyle) fyne.Resource {
	return theme.DefaultTheme().Font(textStyle)
}

func (c *darkTheme) Icon(iconName fyne.ThemeIconName) fyne.Resource {
	return theme.DefaultTheme().Icon(iconName)
}

func (c *darkTheme) Size(sizeName fyne.ThemeSizeName) float32 {
	return theme.DefaultTheme().Size(sizeName)
}
