package config

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// Config holds the two server settings: host and port.
type Config struct {
	Host string `yaml:"host"`
	Port int    `yaml:"port"`
}

func Default() Config {
	return Config{Host: "127.0.0.1", Port: 6379}
}

func (c Config) Addr() string {
	return fmt.Sprintf("%s:%d", c.Host, c.Port)
}

// FromFile loads and parses a YAML config file. Distinct IOError/ParseError
// types let the caller (agni-server's CLI) report read vs. parse failures
// differently.
func FromFile(path string) (Config, error) {
	contents, err := os.ReadFile(path)
	if err != nil {
		return Config{}, &IOError{Err: err}
	}

	cfg := Default()
	if err := yaml.Unmarshal(contents, &cfg); err != nil {
		return Config{}, &ParseError{Err: err}
	}
	return cfg, nil
}

type IOError struct {
	Err error
}

func (e *IOError) Error() string {
	return fmt.Sprintf("could not read config file: %s", e.Err)
}

func (e *IOError) Unwrap() error { return e.Err }

type ParseError struct {
	Err error
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("invalid config: %s", e.Err)
}

func (e *ParseError) Unwrap() error { return e.Err }
