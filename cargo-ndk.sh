#!/bin/sh

set -e

COLOR_RED='\033[0;31m'          # Red
COLOR_GREEN='\033[0;32m'        # Green
COLOR_YELLOW='\033[0;33m'       # Yellow
COLOR_BLUE='\033[0;94m'         # Blue
COLOR_PURPLE='\033[0;35m'       # Purple
COLOR_OFF='\033[0m'             # Reset

success() {
    printf '%b\n' "${COLOR_GREEN}[✔] $*${COLOR_OFF}"
}

die() {
    printf '%b\n' "${COLOR_RED}💔  $*${COLOR_OFF}" >&2
    exit 1
}

run() {
    printf '%b\n' "${COLOR_PURPLE}==>${COLOR_OFF} ${COLOR_GREEN}$*${COLOR_OFF}"
    eval "$*"
    echo
}

getvalue() {
    if [ $# -eq 0 ] ; then
        cut -d= -f2
    else
        echo "$1" | cut -d= -f2
    fi
}

check_if_pie_executable_for_the_given_abi() {
    case $1 in
        armeabi-v7a) file "$2" | grep -q 'ELF 32-bit LSB pie executable, ARM, EABI5'   ;;
        arm64-v8a)   file "$2" | grep -q 'ELF 64-bit LSB pie executable, ARM aarch64,' ;;
        x86)         file "$2" | grep -q 'ELF 32-bit LSB pie executable, Intel 80386,' ;;
        x86_64)      file "$2" | grep -q 'ELF 64-bit LSB pie executable, x86-64,'      ;;
        '')          die "check_if_pie_executable_for_the_given_abi <ABI> <FILEPATH> , <ABI> is not given." ;;
        *)           die "check_if_pie_executable_for_the_given_abi <ABI> <FILEPATH> , unrecognized <ABI> : $1"
    esac
}

build_and_install() {
    CARGO_INSTALL_ARGS=$@

    unset CARGO_PROJECT_DIR
    unset CARGO_INSTALL_DIR

    while [ -n "$1" ]
    do
        case $1 in
            --path)
                shift
                CARGO_PROJECT_DIR="$1"
                if [ -z "$CARGO_PROJECT_DIR" ] ; then
                    die "--path <PATH> , <PATH> must not be empty."
                fi
                if [ ! -d "$CARGO_PROJECT_DIR" ] ; then
                    die "$1 directory not exists."
                fi
                ;;
            --root)
                shift
                CARGO_INSTALL_DIR="$1"
                if [ -z "$CARGO_INSTALL_DIR" ] ; then
                    die "--root <DIR> , <DIR> must not be empty."
                fi
                ;;
            --path=*)
                CARGO_PROJECT_DIR="$(getvalue "$1")"
                if [ -z "$CARGO_PROJECT_DIR" ] ; then
                    die "--path=<PATH> , <PATH> must not be empty."
                fi
                if [ ! -d "$CARGO_PROJECT_DIR" ] ; then
                    die "$CARGO_PROJECT_DIR directory not exists."
                fi
                ;;
            --root=*)
                CARGO_INSTALL_DIR="$(getvalue "$1")"
                if [ -z "$CARGO_INSTALL_DIR" ] ; then
                    die "--root=<DIR> , <DIR> must not be empty."
                fi
                ;;
            *)  if [ -n "$CARGO_PROJECT_DIR" ] && [ -n "$CARGO_INSTALL_DIR" ] ; then
                    break
                fi
        esac
        shift
    done

    if [ -z "$CARGO_PROJECT_DIR" ] ; then
        CARGO_PROJECT_DIR=.
        CARGO_INSTALL_ARGS="$CARGO_INSTALL_ARGS --path $CARGO_PROJECT_DIR"
    fi

    if [ -z "$CARGO_INSTALL_DIR" ] ; then
        CARGO_INSTALL_DIR="$CARGO_PROJECT_DIR/install.d/$ANDROID_ABI"
        CARGO_INSTALL_ARGS="$CARGO_INSTALL_ARGS --root $CARGO_INSTALL_DIR"
    fi

    run grep   --version
    run tree   --version
    run file   --version
    run rustup --version
    run rustc  --version
    run cargo  --version

    # https://github.com/actions/virtual-environments/blob/main/images/linux/Ubuntu2004-Readme.md#environment-variables-3
    # https://docs.github.com/en/actions/learn-github-actions/environment-variables#default-environment-variables
    if [ "$GITHUB_ACTIONS" = true ] ; then
        export ANDROID_NDK_HOME="$ANDROID_NDK_LATEST_HOME"
        export ANDROID_NDK_ROOT="$ANDROID_NDK_LATEST_HOME"
    else
        if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$ANDROID_NDK_ROOT" ] ; then
            die "please set and export ANDROID_NDK_HOME environment"
        elif [ -n "$ANDROID_NDK_HOME" ] ; then
            export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
        else
            export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
        fi
    fi

    run "env | sed -n '/^ANDROID_NDK/p'"

    run cat "$ANDROID_NDK_HOME/source.properties"

    ANDROID_NDK_VERS=$(grep "Pkg.Revision" "$ANDROID_NDK_HOME/source.properties" | cut -d " " -f3)
    ANDROID_NDK_VERS_MAJOR="$(printf '%s\n' "$ANDROID_NDK_VERS" | cut -d. -f1)"

    HOST_OS_TYPE=$(uname | tr A-Z a-z)
    HOST_OS_ARCH=$(uname -m)

    ANDROID_NDK_TOOLCHAIN_DIR=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST_OS_TYPE-$HOST_OS_ARCH

    # https://crates.io/crates/cc
    # https://docs.rs/cc/latest/cc/
    # https://github.com/alexcrichton/cc-rs
    export HOST_CC=cc
    export HOST_CXX=c++
    export HOST_AR=ar

    if [ "$ANDROID_ABI" = 'armeabi-v7a' ] ; then
        EABI=eabi
    else
        EABI=
    fi

    export TARGET_CC=${ANDROID_ARCH}-linux-android${EABI}${ANDROID_MIN_API_LEVEL}-clang
    export TARGET_CXX="${TARGET_CC}++"
    export TARGET_AR=llvm-ar

    export TARGET_CFLAGS="--sysroot $ANDROID_NDK_TOOLCHAIN_DIR/sysroot"
    export TARGET_CXXFLAGS="$TARGET_CFLAGS"

    if [ "$ANDROID_ARCH" = 'armv7a' ] ; then
        export RUST_TARGET=armv7-linux-androideabi
    else
        export RUST_TARGET=${ANDROID_ARCH}-linux-android
    fi

    RUST_TARGET_UPPERCASE_UNDERSCORE=$(printf '%s\n' "$RUST_TARGET" | tr a-z A-Z | tr - _)

    # https://doc.rust-lang.org/cargo/reference/config.html#environment-variables
    # https://doc.rust-lang.org/cargo/reference/environment-variables.html
    export "CARGO_TARGET_${RUST_TARGET_UPPERCASE_UNDERSCORE}_AR"="$TARGET_AR"
    export "CARGO_TARGET_${RUST_TARGET_UPPERCASE_UNDERSCORE}_LINKER"="$TARGET_CC"

    if [ "$ANDROID_NDK_VERS_MAJOR" -ge 23 ] ; then
        # https://github.com/rust-lang/rust/pull/85806
        TEMP_LIBRARY_DIR=$(mktemp -d)
        export RUSTFLAGS="-Clink-arg=-L$TEMP_LIBRARY_DIR"
        echo 'INPUT(-lunwind)' > $TEMP_LIBRARY_DIR/libgcc.a
    fi

    run ls -l "$CARGO_PROJECT_DIR"

    export PATH="$ANDROID_NDK_TOOLCHAIN_DIR/bin:$PATH"

    printf '%b\n' "${COLOR_PURPLE}==>${COLOR_OFF} ${COLOR_GREEN}printf variables${COLOR_OFF}
      HOST_OS_TYPE  = $HOST_OS_TYPE
      HOST_OS_ARCH  = $HOST_OS_ARCH

      HOST_AR       = $HOST_AR
      HOST_CC       = $HOST_CC
      HOST_CXX      = $HOST_CXX
      HOST_CFLAGS   = $HOST_CFLAGS
      HOST_CXXFLAGS = $HOST_CXXFLAGS

    TARGET_AR       = $TARGET_AR
    TARGET_CC       = $TARGET_CC
    TARGET_CXX      = $TARGET_CXX
    TARGET_CFLAGS   = $TARGET_CFLAGS
    TARGET_CXXFLAGS = $TARGET_CXXFLAGS

    ANDROID_ABI     = $ANDROID_ABI
    ANDROID_ARCH    = $ANDROID_ARCH
    ANDROID_MIN_API_LEVEL = $ANDROID_MIN_API_LEVEL
    
    ANDROID_NDK_VERS= $ANDROID_NDK_VERS
    ANDROID_NDK_VERS_MAJOR=$ANDROID_NDK_VERS_MAJOR
    "

    run rustup target add $RUST_TARGET

    # https://doc.rust-lang.org/cargo/commands/cargo-install.html
    run cargo install --target $RUST_TARGET $CARGO_INSTALL_ARGS

    run tree "$CARGO_INSTALL_DIR"

    for item in $(ls "$CARGO_INSTALL_DIR/bin")
    do
        run file "$CARGO_INSTALL_DIR/bin/$item"
        if run check_if_pie_executable_for_the_given_abi "$ANDROID_ABI" "$CARGO_INSTALL_DIR/bin/$item" ; then
            echo true
        else
            echo false
        fi
    done
}

show_help() {
echo "USAGE: ${COLOR_GREEN}./cargo-ndk.sh <ANDROID-ABI> <ANDROID-MIN-API-LEVEL> [cargo install OPTIONS]${COLOR_OFF}

${COLOR_GREEN}ANDROID-ABI${COLOR_OFF}           : one of ${COLOR_PURPLE}armeabi-v7a arm64-v8a x86 x86_64${COLOR_OFF}
${COLOR_GREEN}ANDROID-MIN-API-LEVEL${COLOR_OFF} : https://developer.android.com/studio/releases/platforms


EXAMPLES:

${COLOR_GREEN}./cargo-ndk.sh -h${COLOR_OFF}

${COLOR_GREEN}./cargo-ndk.sh --help${COLOR_OFF}

${COLOR_GREEN}./cargo-ndk.sh arm64-v8a 21${COLOR_OFF}

${COLOR_GREEN}./cargo-ndk.sh arm64-v8a 21 -vv --path . --root ./install.d/arm64-v8a${COLOR_OFF}
"
}

show_usage_and_die() {
    printf '%b\n' "${COLOR_RED}💔  USAGE: ${COLOR_OFF} ${COLOR_GREEN}./cargo-ndk.sh <ANDROID-ABI> <ANDROID-MIN-API-LEVEL> [cargo install OPTIONS]${COLOR_OFF}" >&2
    printf '%b\n' "${COLOR_RED}    $*${COLOR_OFF}" >&2
    exit 1
}

main() {
    case $1 in
        ''|-h|--help) show_help ; exit 0 ;;
    esac

    unset ANDROID_ABI
    unset ANDROID_MIN_API_LEVEL

    case $1 in
        armeabi-v7a|arm64-v8a|x86|x86_64)
            ANDROID_ABI=$1 ;;
        '') show_usage_and_die "<ANDROID-ABI> must not be empty." ;;
        *)  show_usage_and_die "specified <ANDROID-ABI> : $1\n    supported <ANDROID-ABI> : armeabi-v7a arm64-v8a x86 x86_64"
    esac

    case $ANDROID_ABI in
        armeabi-v7a) ANDROID_ARCH='armv7a'  ;;
        arm64-v8a)   ANDROID_ARCH='aarch64' ;;
        x86)         ANDROID_ARCH='i686'    ;;
        x86_64)      ANDROID_ARCH='x86_64'  ;;
    esac

    shift

    case $1 in
        *[!0123456789]*)
            show_usage_and_die "<ANDROID-MIN-API-LEVEL> must be a integer." ;;
        '') show_usage_and_die "<ANDROID-MIN-API-LEVEL> must not be empty." ;;
        *)  ANDROID_MIN_API_LEVEL="$1"
    esac

    shift

    build_and_install $@
}

main $@
