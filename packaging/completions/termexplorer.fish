# Fish completion for termexplorer

# Disable file completions by default unless specified
complete -c termexplorer -f

# Options
complete -c termexplorer -s h -l help -d "Print help information"
complete -c termexplorer -s V -s v -l version -d "Print version information"
complete -c termexplorer -l health -d "Print diagnostic health check report and exit"
complete -c termexplorer -l cwd-file -r -d "Write the last active directory to PATH on exit"
complete -c termexplorer -l chooser-file -r -d "Write the selected file path to PATH on exit"

# Positional argument: directory
complete -c termexplorer -a "(__fish_complete_directories)" -d "Initial directory"
