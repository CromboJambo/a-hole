use std *

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Resolve a path under the crabjar config root (~/.config/crabjar/...)
def crabjar-path [...parts: string] {
    $env.HOME | path join ".config" "crabjar" ...$parts
}

# Ensure a directory exists, creating it (and parents) if needed
def ensure-dir [dir: string] {
    if not ($dir | path exists) {
        mkdir $dir
    }
}

# ---------------------------------------------------------------------------
# Wizard
# ---------------------------------------------------------------------------

# Create and interactively configure a new crabjar agent
def "crabjar wizard" [name: string] {
    print $"Creating new agent named ($name)"

    let agent_dir = (crabjar-path "agents" $name)
    ensure-dir $agent_dir

    let template_file = (crabjar-path "templates" "default_agent.toml")
    ensure-dir (crabjar-path "templates")
    if not ($template_file | path exists) {
        create default template $template_file
    }

    interactive setup $name
}

# ---------------------------------------------------------------------------
# Template
# ---------------------------------------------------------------------------

# Write the default agent TOML template to the given path
def "create default template" [path: string] {
    let template_content = '[agent]
name = "default"
description = "Default agent"

[tools]
enabled = ["shell", "file", "http"]

[config]
max_tokens = 2048
temperature = 0.7

[state]
persist = true
'
    $template_content | save -f $path
}

# ---------------------------------------------------------------------------
# Interactive setup
# ---------------------------------------------------------------------------

# Walk the user through configuring an agent; uses the supplied name as default
def "interactive setup" [agent_name: string] {
    print "=== Crabjar Agent Configuration Wizard ==="

    # Re-use the name passed in; let the user override it
    let raw_name = (input $"Agent Name (default: ($agent_name)): " | str trim)
    let name = if $raw_name == "" { $agent_name } else { $raw_name }

    let desc = (input "Description: " | str trim)

    # Tool selection loop
    mut selected_tools: list<string> = []
    let tool_map = {
        "s": "shell"
        "f": "file"
        "h": "http"
        "g": "git"
        "n": "network"
    }

    loop {
        let choice = (input "Select a tool — [s]hell [f]ile [h]ttp [g]it [n]etwork [d]one: " | str trim)

        if $choice == "d" { break }

        if ($tool_map | get $choice) != null {
            let tool = ($tool_map | get $choice)
            if not ($selected_tools | any { |t| $t == $tool }) {
                $selected_tools = ($selected_tools | append $tool)
                print $"  ✓ added ($tool)"
            } else {
                print $"  ($tool) already selected"
            }
        } else {
            print $"  Unknown option: '($choice)'"
        }
    }

    # Numeric inputs with safe defaults
    let raw_tokens = (input "Max tokens (default 2048): " | str trim)
    let max_tokens = if $raw_tokens == "" { 2048 } else { $raw_tokens | into int }

    let raw_temp = (input "Temperature (default 0.7): " | str trim)
    let temperature = if $raw_temp == "" { 0.7 } else { $raw_temp | into float }

    generate config $name $desc $selected_tools $max_tokens $temperature

    print $"Agent '($name)' created successfully!"
}

# ---------------------------------------------------------------------------
# Config generation
# ---------------------------------------------------------------------------

# Serialise agent parameters to a TOML config file
def "generate config" [
    name: string,
    desc: string,
    tools: list<string>,
    max_tokens: int,
    temperature: float
] {
    let tools_str = ($tools | each { |x| $"\"($x)\"" } | str join ", ")

    let config_content = $"[agent]
name = \"($name)\"
description = \"($desc)\"

[tools]
enabled = [($tools_str)]

[config]
max_tokens = ($max_tokens)
temperature = ($temperature)

[state]
persist = true
"

    let agent_dir = (crabjar-path "agents" $name)
    ensure-dir $agent_dir

    let path = ($agent_dir | path join "config.toml")
    $config_content | save -f $path

    print $"Configuration saved to: ($path)"
}

# ---------------------------------------------------------------------------
# Agent execution
# ---------------------------------------------------------------------------

# Run a configured crabjar agent by name
def "run agent" [agent_name: string] {
    let config_path = (crabjar-path "agents" $agent_name "config.toml")

    if not ($config_path | path exists) {
        print $"Configuration file does not exist: ($config_path)"
        return
    }

    let result = (^crabjar run --config $config_path | complete)

    if $result.exit_code == 0 {
        print "Agent execution successful!"
        print $result.stdout
    } else {
        print $"Error: ($result.stderr)"
    }
}

# ---------------------------------------------------------------------------
# Pipeline  (replaces the old "create workflow" / "create pipeline" pair)
# ---------------------------------------------------------------------------

# Create a named pipeline from a list of steps and persist it to disk.
# Steps may be prefixed with "bash:" or "agent:" to control dispatch.
# NOTE: "bash:" steps are executed verbatim by bash — only use trusted input.
def "create pipeline" [name: string, steps: list<string>] {
    let pipeline = {
        name: $name,
        steps: $steps,
        created_at: (date now),
        status: "active"
    }

    let pipelines_dir = (crabjar-path "pipelines")
    ensure-dir $pipelines_dir

    let path = ($pipelines_dir | path join $"($name | str replace ' ' '_').json")
    $pipeline | to json | save -f $path

    print $"Pipeline '($name)' created at: ($path)"

    $pipeline
}

# Execute each step in a saved pipeline in order
def "run pipeline" [pipeline_name: string] {
    let path = (crabjar-path "pipelines" $"($pipeline_name | str replace ' ' '_').json")

    if not ($path | path exists) {
        print $"Pipeline does not exist: ($path)"
        return
    }

    let pipeline = (open $path)

    for step in $pipeline.steps {
        print $"Executing step: ($step)"

        if ($step | str starts-with "bash:") {
            # WARNING: the command string is passed directly to bash.
            # Only run pipelines sourced from trusted input.
            let cmd = ($step | str substring 5..)
            let result = (^bash -c $cmd | complete)
            if $result.exit_code != 0 {
                print $"Step failed \(exit ($result.exit_code)\): ($result.stderr)"
            } else {
                print $result.stdout
            }
        } else if ($step | str starts-with "agent:") {
            let agent_name = ($step | str substring 6..)
            run agent $agent_name
        } else {
            print $"Unknown step prefix, skipping: ($step)"
        }
    }

    print $"Pipeline '($pipeline_name)' completed."
}

# ---------------------------------------------------------------------------
# Tool availability checks
# ---------------------------------------------------------------------------

# Check whether a named tool is available and usable in the current environment
def "tool setup" [tool_name: string] {
    match $tool_name {
        "shell" => {
            let result = (^which sh | complete)
            if $result.exit_code == 0 {
                print $"Shell tool available: ($result.stdout | str trim)"
            } else {
                print "Warning: sh not found — some features may be limited"
            }
        },
        "git" => {
            # First confirm we are inside a git repository at all
            let repo_check = (^git rev-parse --is-inside-work-tree | complete)
            if $repo_check.exit_code != 0 {
                print "No git repository found in the current directory"
            } else {
                let status = (^git status --porcelain | complete)
                if ($status.stdout | str trim) == "" {
                    print "Git repository found — working tree is clean"
                } else {
                    print $"Git repository found — uncommitted changes present:\n($status.stdout | str trim)"
                }
            }
        },
        _ => {
            print $"Tool setup for '($tool_name)' not implemented yet."
        }
    }
}
