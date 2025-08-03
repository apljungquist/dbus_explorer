# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a D-Bus Explorer web application built in Rust that provides a web interface for browsing and exploring D-Bus services, objects, and interfaces on the system. It's designed as an ACAP (Axis Camera Application Platform) application for embedded Linux systems.

## Architecture

The project consists of two main Rust modules:

- **`src/main.rs`**: Axum web server that serves the HTML interface and provides REST API endpoints for D-Bus exploration
- **`src/dbus_introspection.rs`**: Core D-Bus introspection logic that connects to the system D-Bus, queries services, and parses XML introspection data

### Web Interface Structure
- Landing page lists all D-Bus services
- Service-specific pages show objects in flat list format with full paths and interface details
- Object-specific pages show detailed interface information (methods, properties, signals) and links to child objects
- All pages include D-Bus type reference documentation

### D-Bus Integration
- Uses the `dbus` crate with system bus connection
- Implements recursive object tree exploration
- Parses XML introspection data using `quick-xml` with serde
- Filters out standard freedesktop interfaces to reduce noise
- Handles access denied and authorization errors gracefully

## Code Quality Requirements

**MANDATORY**: Before completing any code changes, you MUST run `cargo check` (or the appropriate verification command for the project) to ensure the code compiles without errors. Do not mark tasks as complete or return control to the user until compilation is verified.

For this Rust project, always run:
- `cargo check` - Fast compilation check
- `make fix_format` - Automatically fix code formatting issues

**For comprehensive verification before major commits:**
- `make check` - Runs complete CI pipeline (build, docs, format, lint, tests, file consistency)
- `make fix_lint` - Automatically fix linting issues when needed

## Common Development Commands

### Building and Testing
```bash
# Run all checks (build, docs, format, lint, tests, file consistency)
make check

# Individual check targets
make check_build          # Build verification
make check_tests          # Run tests
make check_docs           # Documentation verification
make check_format         # Formatting verification
make check_lint           # Linting verification
make check_generated_files # File consistency verification
```

### Code Quality and Fixes
```bash
# Automatically fix issues
make fix_format           # Fix code formatting
make fix_lint             # Fix linting issues

# Or use individual commands
cargo build --locked --workspace
cargo test --all-targets --locked --workspace
cargo clippy --fix
```

### Device Deployment (ACAP SDK)
The application is designed to run on separate Axis devices. Use these commands for device deployment:

```bash
# Build and deploy to device
cargo-acap-sdk build    # Build app with release profile
cargo-acap-sdk install  # Build and install on device
cargo-acap-sdk run      # Build and run on device (app must not be running)

# Device management
cargo-acap-sdk start    # Start app on device
cargo-acap-sdk stop     # Stop app on device
cargo-acap-sdk restart  # Restart app on device
cargo-acap-sdk remove   # Remove app from device

# Testing
cargo-acap-sdk test     # Build in test mode and run on device (app must not be running)
cargo-acap-sdk run      # Good for testing app startup without crashes
```

**Important**: Before using `cargo-acap-sdk run` or `cargo-acap-sdk test`, ensure the app is not already running:
- If started with `cargo-acap-sdk start`, use `cargo-acap-sdk stop` first
- If running from a previous command, kill that process before proceeding

**Testing Tip**: Use `cargo-acap-sdk run` to verify the application starts without crashes and handles initialization properly. This provides immediate feedback on startup issues, configuration problems, or runtime panics.

### SSH Utilities for Device Interaction
For advanced device interaction, use `acap-ssh-utils` (requires SSH setup on device):

```bash
# Set device connection (environment variables)
export AXIS_DEVICE_IP=<device_ip>
export AXIS_DEVICE_USER=root
export AXIS_DEVICE_PASS=<password>

# Patch and run commands (app must not be running)
acap-ssh-utils --host $AXIS_DEVICE_IP patch      # Patch app on device
acap-ssh-utils --host $AXIS_DEVICE_IP run-app    # Run app with terminal output
acap-ssh-utils --host $AXIS_DEVICE_IP run-other  # Run any executable on device
```

**Important**: Before using any `acap-ssh-utils` commands, ensure the app is stopped using `cargo-acap-sdk stop` or kill any background processes.

### ACAP Configuration
The application is configured as an ACAP package in `manifest.json` with:
- HTTP proxy configuration with `"apiPath": "app"` routing to port 2001
- Settings page configured as `"settingPage": "app"` pointing to the dynamic landing page
- Simplified configuration without static files

### Development Environment
- Rust toolchain 1.88.0 specified in `rust-toolchain.toml`
- Targets ARM architectures: `aarch64-unknown-linux-gnu` and `thumbv7neon-unknown-linux-gnueabihf`
- Development container configured with Rust analyzer and debugging tools

## Server Configuration

The web server runs locally on the device at `127.0.0.1:2001`, but is accessed externally through the device's IP address via HTTP proxy configuration.

### Accessing the Application
- **Local (on device)**: `http://127.0.0.1:2001/local/dbus_explorer/app`
- **External (from development machine)**: `http://$AXIS_DEVICE_IP/local/dbus_explorer/app`

### API Endpoints
All endpoints are accessible both locally and externally:
- `/local/dbus_explorer/app` - Landing page with service list
- `/local/dbus_explorer/app/all` - Flat view of all services and objects
- `/local/dbus_explorer/app/service/{service_name}` - Service-specific page
- `/local/dbus_explorer/app/service/{service_name}/{object_path}` - Object-specific page

## Key Dependencies

- **axum**: Web framework for HTTP server
- **dbus**: System D-Bus communication
- **quick-xml + serde**: XML parsing for introspection data
- **tokio**: Async runtime
- **anyhow**: Error handling
- **acap-logging**: ACAP-specific logging
- **thiserror**: Custom error type generation

## ACAP Development Resources

For comprehensive ACAP app development with Rust, refer to these official resources:

### Primary Documentation
- **ACAP Rust Repository**: https://github.com/AxisCommunications/acap-rs/tree/main
  - Contains examples, templates, and comprehensive documentation
  - Source code for `cargo-acap-sdk` and `acap-ssh-utils`
  - Best practices and development patterns

### Axis Developer Portal
- **Main Developer Site**: https://developer.axis.com
- **ACAP Documentation**: https://developer.axis.com/acap/
  - ACAP architecture and concepts
  - Development environment setup
  - Deployment and lifecycle management
- **VAPIX API Documentation**: https://developer.axis.com/vapix/
  - Device communication APIs
  - System integration interfaces

These resources provide essential context for:
- ACAP application architecture and constraints
- Device-specific APIs and capabilities  
- Best practices for embedded development
- Integration with Axis camera platform features