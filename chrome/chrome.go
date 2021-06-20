package chrome

type Chrome struct {
	Resolution       string
	ChromeTimeout    int
	ChromeTimeBudget int
	Path             string
	UserAgent        string
	Argvs            []string
	ScreenshotPath   string
}
