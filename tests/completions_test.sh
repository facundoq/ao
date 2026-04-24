#!/usr/bin/env bash
set -e

EXE="./target/debug/ao"

if [ ! -f "$EXE" ]; then
    echo "Binary not found at $EXE. Please run cargo build first."
    exit 1
fi

echo "--- Testing Bash Completions ---"
BASH_SCRIPT=$($EXE self completions generate bash)

test_bash() {
    local cmd="$1"
    local expected="$2"
    echo "Testing Bash: '$cmd' -> expects '$expected'"
    
    bash <<BASH_EOF
        _get_comp_words_by_ref() {
            cur="\${COMP_WORDS[COMP_CWORD]}"
            prev="\${COMP_WORDS[COMP_CWORD-1]}"
            words=("\${COMP_WORDS[@]}")
            cword=\$COMP_CWORD
        }
        $BASH_SCRIPT
        read -a COMP_WORDS <<< "$cmd"
        [[ "$cmd" == *" " ]] && COMP_WORDS+=("")
        COMP_CWORD=\$((\${#COMP_WORDS[@]} - 1))
        COMP_LINE="$cmd"
        _ao "ao" "\${COMP_WORDS[\$COMP_CWORD]}" "\${COMP_WORDS[\$COMP_CWORD-1]}"
        
        for item in "\${COMPREPLY[@]}"; do
            if [[ "\$item" == *"\$expected"* ]]; then exit 0; fi
        done
        echo "FAILURE: Suggestions were: \${COMPREPLY[*]}"
        exit 1
BASH_EOF
}

# Bash Tests
test_bash "ao group del " "root"
test_bash "ao group mod " "root"
test_bash "ao user del " "root"
test_bash "ao user passwd " "root"
test_bash "ao user mod " "root"
test_bash "ao user mod root " "shell"
test_bash "ao user mod root shell " "/bin/"
test_bash "ao self completions generate " "bash"
test_bash "ao self completions setup " "zsh"
test_bash "ao service status " ".service"
test_bash "ao system power " "reboot"
test_bash "ao log tail " ".service"
test_bash "ao distribution " "upgrade"
test_bash "ao network " "interfaces"
test_bash "ao network fw " "status"
test_bash "ao network wifi " "scan"
test_bash "ao boot " "ls"
test_bash "ao boot mod " "ls"
test_bash "ao gui " "info"
test_bash "ao gui display " "ls"
test_bash "ao device " "ls"
test_bash "ao device bt " "scan"
test_bash "ao virtualization " "ls"
test_bash "ao virtualization start " "help"
test_bash "ao security " "audit"

echo ""
echo "--- Testing Zsh Completions ---"

test_zsh() {
    local cmd_args=($1)
    local expected="$2"
    local index=$3
    echo "Testing Zsh: '${cmd_args[*]} ' -> expects '$expected' (Index: $index)"
    
    # Simulate the Zsh bridge call
    local output
    output=$(AO_COMPLETE=zsh _CLAP_COMPLETE_INDEX=$index "$EXE" -- "${cmd_args[@]}" "")
    
    if echo "$output" | grep -q "$expected"; then
        echo "SUCCESS"
    else
        echo "FAILURE: Output was: $output"
        exit 1
    fi
}

# Zsh Tests
# "ao group del " -> index 3 (0:ao, 1:group, 2:del, 3:"")
test_zsh "ao group del" "root" 3
test_zsh "ao group mod" "root" 3
test_zsh "ao user del" "root" 3
test_zsh "ao user passwd" "root" 3
test_zsh "ao user mod" "root" 3
# "ao user mod root " -> index 4
test_zsh "ao user mod root" "shell" 4
# "ao user mod root shell " -> index 5
test_zsh "ao user mod root shell" "/bin/" 5
test_zsh "ao self completions generate" "bash" 4
test_zsh "ao self completions setup" "zsh" 4
test_zsh "ao service status" ".service" 3
test_zsh "ao system power" "reboot" 3
test_zsh "ao log tail" ".service" 3
test_zsh "ao distribution" "upgrade" 2
test_zsh "ao network" "interfaces" 2
test_zsh "ao network fw" "status" 3
test_zsh "ao network wifi" "scan" 3
test_zsh "ao boot" "ls" 2
test_zsh "ao boot mod" "ls" 3
test_zsh "ao gui" "info" 2
test_zsh "ao gui display" "ls" 3
test_zsh "ao device" "ls" 2
test_zsh "ao device bt" "scan" 3
test_zsh "ao virtualization" "ls" 2
test_zsh "ao virtualization start" "help" 3
test_zsh "ao security" "audit" 2

echo ""
echo "All completion tests passed!"
