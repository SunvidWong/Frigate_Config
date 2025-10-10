module github.com/frigate-config-tool/agent

go 1.21

// Standard library only - no external dependencies
// This keeps the agent binary small and portable

require github.com/stretchr/testify v1.11.1

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)
