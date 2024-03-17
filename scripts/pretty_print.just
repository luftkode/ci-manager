#!/usr/bin/env -S just --justfile
## Cheatsheet: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors

## Contains utilities for printing colored text to the terminal for distro-builder-common
##
## NOTE: the "PRINT" variable is defined by distro-template so here we use "ECHO" instead to avoid multiple definitions.
##
## The "ECHO" variable is defined in the top-level justfile
##  and is an absolute path to this file:
## ECHO := join(justfile_directory(), "util-scripts/just-util/pretty_print.just")
##  thus it can be used to call the print function from any justfile in the project, in any directory,
##  making it immune to cd-ing around in Bash/Python/etc. recipes.
##
## Usage:
##    {{ECHO}} green "Success!"
##    {{ECHO}} cyan "Info"
## By default, a newline is appended to the end of the text. To suppress this, set the NEWLINE variable to something other than "true"
##    {{ECHO}} NEWLINE="no" green "Success!"
##

ANSI_ESC_CLR := '\x1b[0m'

ANSI_BOLD_GREEN := '\x1b[1;32m'
ANSI_BOLD_CYAN := '\x1b[1;36m'
ANSI_BOLD_YELLOW := '\x1b[1;33m'
ANSI_BOLD_RED := '\x1b[1;31m'
ANSI_BOLD_MAGENTA := '\x1b[1;35m'
ANSI_BOLD_BLUE := '\x1b[1;34m'

NEWLINE := 'true'

green TEXT: (print ANSI_BOLD_GREEN TEXT)
cyan TEXT: (print ANSI_BOLD_CYAN TEXT)
yellow TEXT: (print ANSI_BOLD_YELLOW TEXT)
red TEXT: (print ANSI_BOLD_RED TEXT)
magenta TEXT: (print ANSI_BOLD_MAGENTA TEXT)
blue TEXT: (print ANSI_BOLD_BLUE TEXT)

# Generic print function
[private]
print ANSI_START TEXT:
	#!/usr/bin/env bash
	declare -r optional_newline=$( [ {{NEWLINE}} == 'true' ] && echo "\n" || echo "" )
	printf "%b%b%b${optional_newline}" "{{ANSI_START}}" "{{TEXT}}" "{{ANSI_ESC_CLR}}"
