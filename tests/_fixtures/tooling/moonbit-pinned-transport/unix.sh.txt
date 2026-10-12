# taken from bun.sh/install
Color_Off=''

# Regular Colors
Red=''
Green=''
Dim='' # White

# Bold
Bold_White=''
Bold_Green=''

if [[ -t 1 ]]; then
    # Reset
    Color_Off='\033[0m' # Text Reset

    # Regular Colors
    Red='\033[0;31m'   # Red
    Green='\033[0;32m' # Green
    Dim='\033[0;2m'    # White

    # Bold
    Bold_Green='\033[1;32m' # Bold Green
    Bold_White='\033[1m'    # Bold White
fi

error() {
  echo -e "${Red}error${Color_Off}: $*" >&2
  exit 1
}

info() {
  echo -e "${Dim}$* ${Color_Off}"
}

info_bold() {
  echo -e "${Bold_White}$* ${Color_Off}"
}

success() {
  echo -e "${Green}$* ${Color_Off}"
}

ARGUMENTS=()

for i in "$@"; do
  case "$i" in
    -h|--help)
      HELP=true
      ;;
    # -y|--yes)
    #   YES=true
    #   ;;
    # -n|--no)
    #   NO=true
    #   ;;
    -*)
      error "Unknown option: $i"
      ;;
    *)
      ARGUMENTS+=("$i")
      ;;
  esac
done

if [[ $HELP ]]; then
  info_bold "Usage: $0 [options] [arguments]"
  echo
  echo "Options:"
  echo "  -h, --help  Show this help message and exit"
  # echo "  -y, --yes   always choose yes in confirmation prompt"
  # echo "  -n, --no    always choose no in confirmation prompt"
  echo
  echo "Arguments:"
  echo "  None        install the latest version of moonbit"
  echo "  [VERSION]   install the specified version of moonbit"
  exit 0
fi

if [[ ${#ARGUMENTS[@]} -gt 1 ]]; then
  error "Too many arguments"
fi

target=''

case $(uname -ms) in
'Darwin arm64')
    target=darwin-aarch64
    ;;
'Linux x86_64')
    target=linux-x86_64
    ;;
'Linux aarch64')
    target=linux-aarch64
esac

if [[ -z $target ]]; then
  error "Unsupported platform: $(uname -ms)"
fi

if [[ -n $MOONBIT_INSTALL_DEV ]]; then
  target="$target-dev"
fi

version=${ARGUMENTS[0]:-${MOONBIT_INSTALL_VERSION:-latest}}
version=${version//+/%2B}

CLI_MOONBIT="https://cli.moonbitlang.com"

moonbit_uri="$CLI_MOONBIT/binaries/$version/moonbit-$target.tar.gz"
core_uri="$CLI_MOONBIT/cores/core-$version.tar.gz"

moon_home="${MOON_HOME:-$HOME/.moon}"
bin_dir=$moon_home/bin
exe=$bin_dir/moon
moonbit_dest=$HOME/moonbit.tar.gz
lib_dir=$moon_home/lib
core_dest=$lib_dir/core.tar.gz

if [[ -z $moon_home ]]; then
  error "MOON_HOME is not set"
fi

mkdir -p "$moon_home" ||
  error "Failed to create directory \"$moon_home\""

echo "Downloading moonbit ..."
curl --fail --location --progress-bar --output "$moonbit_dest" "$moonbit_uri" ||
  error "Failed to download moonbit from \"$moonbit_uri\""

rm -rf "${moon_home:?}/lib" ||
  error "Failed to remove existing moonbit libraries"

rm -rf "${moon_home:?}/include" ||
  error "Failed to remove existing moonbit includes"

tar xf "$moonbit_dest" --directory="$moon_home" ||
  error "Failed to extract moonbit to \"$moon_home\""

ln -sfn moon "$bin_dir/moonx" ||
  error "Failed to link \"$bin_dir/moonx\" to \"$exe\""

rm -f "$moonbit_dest" ||
  error "Failed to remove \"$moonbit_dest\""

pushd "$bin_dir" >/dev/null || error "Failed to change directory to \"$bin_dir\""
  for i in *; do
    chmod +x "$i" ||
      error "Failed to make \"$i\" executable"
  done
  chmod +x ./internal/tcc ||
    error "Failed to make tcc executable"
popd >/dev/null || error "Failed to change directory to previous directory"

rm -rf "$lib_dir/core" ||
  error "Failed to remove existing core"

echo "Downloading core ..."
curl --fail --location --progress-bar --output "$core_dest" "$core_uri" ||
  error "Failed to download core from \"$core_uri\""

tar xf "$core_dest" --directory="$lib_dir" ||
  error "Failed to extract core to \"$lib_dir\""

rm -f "$core_dest" ||
  error "Failed to remove \"$core_dest\""

echo "Bundling core ..."

PATH=$bin_dir $exe -C "$lib_dir"/core bundle --warn-list -a --all ||
  error "Failed to bundle core"

if [[ $version == "nightly" ]]; then
  PATH=$bin_dir $exe -C "$lib_dir"/core bundle --warn-list -a --target llvm ||
    error "Failed to bundle core for llvm backend"
fi

PATH=$bin_dir $exe -C "$lib_dir"/core bundle --warn-list -a --target wasm-gc --quiet ||
  error "Failed to bundle core to wasm-gc"

tildify() {
  if [[ $1 = $HOME/* ]]; then
    local replacement=\~/
      echo "${1/$HOME\//$replacement}"
    else
      echo "$1"
  fi
}

success "moonbit was installed successfully to $Bold_Green$(tildify "$moon_home")"

echo "To verify the downloaded binaries, check https://www.moonbitlang.com/download#verifying-binaries for instructions."

echo "To know how to add shell completions, run 'moon shell-completion --help'"

if command -v moon >/dev/null 2>&1; then
  echo "Run 'moon help' to get started"
  exit 0
fi

tilde_bin_dir=$(tildify "$bin_dir")
bin_env=${bin_dir//\"/\\\"}
refresh_command=''

if [[ $bin_env = $HOME/* ]]; then
  bin_env=${bin_env/$HOME\//\$HOME/}
fi
quoted_new_path_env="\"$bin_env:\$PATH\""

case $(basename "$SHELL") in
fish)
  commands=(
    "fish_add_path \"$bin_env\""
  )

  fish_config=$HOME/.config/fish/config.fish
  tilde_fish_config=$(tildify "$fish_config")

  if [[ -w $fish_config ]]; then
    {
      echo -e "\n# moonbit"
      for cmd in "${commands[@]}"; do
        echo "$cmd"
      done
    } >> "$fish_config"

    info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_fish_config\""
    refresh_command="source $tilde_fish_config"
  else
    echo "Manually add the directory to $tilde_fish_config (or similar):"

    for command in "${commands[@]}"; do
      info_bold "  $command"
    done
  fi
  ;;
zsh)
  commands=(
    "export PATH=$quoted_new_path_env"
  )
  zsh_config=$HOME/.zshrc
  tilde_zsh_config=$(tildify "$zsh_config")

  if [[ -w $zsh_config ]]; then
    {
      echo -e '\n# moonbit'

      for command in "${commands[@]}"; do
        echo "$command"
      done
    } >>"$zsh_config"

    info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_zsh_config\""

    refresh_command="source $tilde_zsh_config"
  else
    echo "Manually add the directory to $tilde_zsh_config (or similar):"

    for command in "${commands[@]}"; do
      info_bold "  $command"
    done
  fi
  ;;
bash)
  commands=(
    "export PATH=$quoted_new_path_env"
  )
  bash_config=$HOME/.bashrc
  tilde_bash_config=$(tildify "$bash_config")

  if [[ -w $bash_config ]]; then
    {
      echo -e '\n# moonbit'

      for command in "${commands[@]}"; do
        echo "$command"
      done
    } >>"$bash_config"

    info "Added \"$tilde_bin_dir\" to \$PATH in \"$tilde_bash_config\""

    refresh_command="source $tilde_bash_config"
  else
    echo "Manually add the directory to $tilde_bash_config (or similar):"

    for command in "${commands[@]}"; do
      info_bold "  $command"
    done
  fi
  ;;
esac

info "To get started, run:"

if [[ $refresh_command ]]; then
  info_bold "  $refresh_command"
fi

info_bold "  moon help"
