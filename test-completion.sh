#!/bin/zsh
# Test shell completions for opencode-container
# Run on macOS: zsh test-completion.sh

SCRIPT_DIR="${0:A:h}"
PASS=0
FAIL=0

ZSH_COMPLETION=$("${SCRIPT_DIR}/bin/opencode-container" completion --zsh)
BASH_COMPLETION=$("${SCRIPT_DIR}/bin/opencode-container" completion --bash)

run_zsh_test() {
  local desc="$1"
  local expected="$2"
  shift 2

  local tmpfile
  tmpfile=$(mktemp)

  # Build properly quoted words array
  local words_str=""
  for w in "$@"; do
    words_str+="${(q)w} "
  done

  cat > "$tmpfile" <<ZSHEOF
${ZSH_COMPLETION}

# Replace compadd with a recorder (compadd requires completion context)
_test_completions=()
compadd() { _test_completions+=("\$@") }

local -a reply
local -a state state_descr line
local curcontext=':completion::complete:opencode-container:'

words=( ${words_str} )
CURRENT=\$#words
_opencode_container
print -l \${_test_completions[@]}
ZSHEOF

  local result
  result=$(zsh "$tmpfile" 2>&1)
  rm -f "$tmpfile"

  if echo "$result" | grep -Fq -- "$expected"; then
    echo "PASS  zsh  $desc"
    ((PASS++))
  else
    echo "FAIL  zsh  $desc"
    echo "       expected: $expected"
    echo "       got:      $result"
    ((FAIL++))
  fi
}

run_bash_test() {
  local desc="$1"
  local expected="$2"
  shift 2

  local tmpfile
  tmpfile=$(mktemp)

  # Build properly quoted words array for bash
  local words_str=""
  for w in "$@"; do
    if [[ -z "$w" ]]; then
      words_str+="'' "
    else
      words_str+="'$w' "
    fi
  done

  cat > "$tmpfile" <<BASHEOF
#!/bin/bash
${BASH_COMPLETION}

COMP_WORDS=( ${words_str} )
COMP_CWORD=\$(( \${#COMP_WORDS[@]} - 1 ))
COMP_LINE="\${COMP_WORDS[*]}"
COMP_POINT=\${#COMP_LINE}
_opencode_container
printf '%s\n' \${COMPREPLY[@]}
BASHEOF

  local result
  result=$(bash "$tmpfile" 2>&1)
  rm -f "$tmpfile"

  if echo "$result" | grep -Fq -- "$expected"; then
    echo "PASS  bash  $desc"
    ((PASS++))
  else
    echo "FAIL  bash  $desc"
    echo "        expected: $expected"
    echo "        got:      $result"
    ((FAIL++))
  fi
}

echo "--- zsh completion ---"

run_zsh_test "no subcommand: shows web" "web" opencode-container ""
run_zsh_test "no subcommand: shows tui" "tui" opencode-container ""
run_zsh_test "no subcommand: shows completion" "completion" opencode-container ""
run_zsh_test "no subcommand: shows --build" "--build" opencode-container ""
run_zsh_test "no subcommand: shows --help" "--help" opencode-container ""
run_zsh_test "after web: shows --port" "--port" opencode-container web ""
run_zsh_test "after web: shows --no-open" "--no-open" opencode-container web ""
run_zsh_test "after completion: shows --bash" "--bash" opencode-container completion ""
run_zsh_test "after completion: shows --zsh" "--zsh" opencode-container completion ""
run_zsh_test "after tui: no completions" "" opencode-container tui ""
run_zsh_test "after projects: no completions" "" opencode-container projects ""

echo ""
echo "--- bash completion ---"

run_bash_test "no subcommand: shows web" "web" opencode-container ""
run_bash_test "no subcommand: shows tui" "tui" opencode-container ""
run_bash_test "no subcommand: shows completion" "completion" opencode-container ""
run_bash_test "no subcommand: shows --build" "--build" opencode-container ""
run_bash_test "no subcommand: shows --help" "--help" opencode-container ""
run_bash_test "after web: shows --port" "--port" opencode-container web ""
run_bash_test "after web: shows --no-open" "--no-open" opencode-container web ""
run_bash_test "after completion: shows --bash" "--bash" opencode-container completion ""
run_bash_test "after completion: shows --zsh" "--zsh" opencode-container completion ""
run_bash_test "after tui: no completions" "" opencode-container tui ""
run_bash_test "after projects: no completions" "" opencode-container projects ""

echo ""
echo "Results: $PASS passed, $FAIL failed"
exit $((FAIL > 0 ? 1 : 0))
