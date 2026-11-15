# Bash completion for termexplorer
_termexplorer() {
    local cur prev words cword
    _init_completion || return

    local opts="-h --help -V -v --version --health --cwd-file --chooser-file"

    case "$prev" in
        --cwd-file|--chooser-file)
            _filedir
            return 0
            ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "$opts" -- "$cur") )
        return 0
    fi

    _filedir -d
}

complete -F _termexplorer termexplorer
