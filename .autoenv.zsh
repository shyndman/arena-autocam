# Defines a shortcut to our task runner.
#
# This will automatically install shell completion functions if missing or
# the build changed.
aatask() {
  workspace_path="$(dirname "$0")"
  # TODO(shyndman): Figure out how to grab the path to the current arch
  bin_path="$workspace_path/target/x86_64-unknown-linux-gnu/debug/aatask"
  build_stderr_path=$(mktemp)

  if ! cargo build --color=always --bin=aatask 2> $build_stderr_path
  then
    cat $build_stderr_path
    return
  fi

  grep --quiet "rustc " $build_stderr_path
  no_build=$?
  whence -f _aatask > /dev/null
  missing_completer=$?
  if [ $no_build -eq 0 -o $missing_completer -eq 1 ]
  then
    if [ $missing_completer -eq 0 ]
    then
      compdef -d aatask;
      unfunction -m '_aatask*';
    fi

    source <($bin_path generate-completion-script 2> /dev/null)
    compdef _aatask aatask
  fi

  $bin_path $@
}

aatask > /dev/null 2> /dev/null
