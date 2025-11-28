#!/usr/bin/env bash
# Development helper script for PacMagic

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function print_header() {
    echo -e "${BOLD}${GREEN}==> $1${NC}"
}

function print_warning() {
    echo -e "${YELLOW}Warning: $1${NC}"
}

function check_rust() {
    if ! command -v cargo &> /dev/null; then
        echo "Rust is not installed!"
        echo "Install from: https://rustup.rs/"
        exit 1
    fi
    print_header "Rust version: $(rustc --version)"
}

function setup() {
    print_header "Setting up PacMagic development environment"
    check_rust
    
    print_header "Installing development tools"
    rustup component add rustfmt clippy
    
    print_header "Building project"
    cargo build
    
    echo ""
    echo -e "${GREEN}Setup complete!${NC}"
    echo "Run './dev.sh help' for available commands"
}

function run_tests() {
    print_header "Running tests"
    cargo test "$@"
}

function run_lint() {
    print_header "Running linter (clippy)"
    cargo clippy -- -D warnings
}

function run_format() {
    print_header "Formatting code"
    cargo fmt
}

function run_check() {
    print_header "Checking code"
    cargo check --all-features
}

function build_release() {
    print_header "Building release binary"
    cargo build --release
    echo ""
    echo -e "${GREEN}Release binary:${NC} target/release/pacmagic"
}

function run_dev() {
    print_header "Running PacMagic (dev mode)"
    cargo run -- "$@"
}

function run_clean() {
    print_header "Cleaning build artifacts"
    cargo clean
}

function run_doc() {
    print_header "Building documentation"
    cargo doc --open --no-deps
}

function run_all_checks() {
    print_header "Running all checks"
    run_format
    run_check
    run_lint
    run_tests
    echo ""
    echo -e "${GREEN}All checks passed!${NC}"
}

function show_help() {
    cat << EOF
PacMagic Development Helper

Usage: ./dev.sh <command> [args]

Commands:
    setup           Set up development environment
    test [args]     Run tests (pass additional args to cargo test)
    lint            Run clippy linter
    format          Format code with rustfmt
    check           Check code without building
    build           Build release binary
    run [args]      Run PacMagic in dev mode
    clean           Clean build artifacts
    doc             Build and open documentation
    all             Run all checks (format, lint, test)
    help            Show this help message

Examples:
    ./dev.sh setup
    ./dev.sh test
    ./dev.sh run -- detect
    ./dev.sh run -- install --dry-run neovim
    ./dev.sh all

EOF
}

# Main script logic
case "${1:-help}" in
    setup)
        setup
        ;;
    test)
        shift
        run_tests "$@"
        ;;
    lint)
        run_lint
        ;;
    format)
        run_format
        ;;
    check)
        run_check
        ;;
    build)
        build_release
        ;;
    run)
        shift
        run_dev "$@"
        ;;
    clean)
        run_clean
        ;;
    doc)
        run_doc
        ;;
    all)
        run_all_checks
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run './dev.sh help' for usage"
        exit 1
        ;;
esac


