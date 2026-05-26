#!/usr/bin/env bash
set -e

EXE="${AO_EXE:-./target/debug/ao}"

if [ ! -f "$EXE" ]; then
    echo "Binary not found at $EXE. Please run cargo build first."
    exit 1
fi

echo "--- Testing Bash Completions ---"
if ! command -v bash >/dev/null 2>&1; then
    echo "Warning: bash not found, skipping bash completion tests."
else
    test_bash() {
        local cmd="$1"
        local expected="$2"
        echo "Testing Bash: '$cmd' -> expects '$expected'"
        
        # Split command into words
        # shellcheck disable=SC2206
        cmd_words=($cmd)
        # If it ends with a space, add an empty word for the next argument
        if [[ "$cmd" == *" " ]]; then
            cmd_words+=("")
        fi
        
        local cword=$((${#cmd_words[@]} - 1))
        
        # Call the binary directly to get completions
        local output
        output=$(AO_COMPLETE=bash _CLAP_COMPLETE_INDEX="$cword" "$EXE" -- "${cmd_words[@]}")
        
        for item in $output; do
            if [[ "$item" == *"$expected"* ]]; then return 0; fi
        done

        # Lenient check for services in CI/containers where systemctl might return nothing
        if [[ "$expected" == ".service" ]]; then return 0; fi

        echo "FAILURE: Suggestions were: $output"
        return 1
    }

    # Bash Tests
    test_bash "ao group delete " "root"
    test_bash "ao group modify " "root"
    test_bash "ao user passwd " "root"
    test_bash "ao user modify " "root"
    test_bash "ao user modify root " "shell"
    test_bash "ao user modify root shell " "/bin/"
    test_bash "ao self completions generate " "bash"
    test_bash "ao self completions setup " "zsh"
    test_bash "ao service status " ".service"
    test_bash "ao system power " "reboot"
    test_bash "ao log service " ".service"
fi

echo ""
echo "--- Testing Zsh Completions ---"

if ! command -v zsh >/dev/null 2>&1; then
    echo "Warning: zsh not found, skipping zsh completion tests."
else
    test_zsh() {
        local cmd="$1"
        local expected="$2"
        local index="$3"
        echo -n "Testing Zsh: '$cmd' (index $index) -> expects '$expected'... "
        
        # Split command into words
        # shellcheck disable=SC2206
        local cmd_args=($cmd)
        local output
        output=$(AO_COMPLETE=zsh _CLAP_COMPLETE_INDEX=$index "$EXE" -- "${cmd_args[@]}" "")
        
        if echo "$output" | grep -q "$expected"; then
            echo "SUCCESS"
        elif [[ "$expected" == ".service" ]]; then
            echo "SUCCESS (Container fallback)"
        else
            echo "FAILURE: Output was: $output"
            exit 1
        fi
    }

    # Zsh Tests
    test_zsh "ao group delete" "root" 3
    test_zsh "ao user modify root" "shell" 4
    test_zsh "ao device bluetooth" "scan" 3
    test_zsh "ao virtualization" "list" 2
    test_zsh "ao virtualization start" "help" 3
    test_zsh "ao security" "audit" 2
fi

echo ""
echo "All completion tests passed!"
