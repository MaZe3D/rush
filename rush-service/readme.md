# rush - remote shell for microcontrollers
rush is a simple shell to interact with the GPIO-Pins on an ESP32-S3 microcontroller over WIFI.

## Build Envoirement
Setting up the build-environment is possible for Windows and Linux.

### Prerequisites on Windows
On Windows (version 10 and 11 respectevly) you need to follow these steps:
1. You need a supported Linker on your system.
   If you have a Linker installed, you can ignore this step.
   For Windows we recommend the [Microsoft Visual Studio Build Tools](), which can be obtained via winget:
   ```
   winget install Microsoft.VisualStudio.2022.BuildTools --interactive
   ```

2. Inside the Visual Studio Installer select the `C++ Development Tools`

3. Install the rust-toolchain installer [rustup](https://rustup.rs/).
   This can be done by downloading it from the official site or via winget via the command:
   ```
   winget install Rustlang.Rustup
   ```

4. You can continue with [Setting up the Toolchain](#setting-up-the-toolchain).

### Prerequisites on Linux
The following steps are made for Ubuntu and other Debian-Based Distributoions.
The exact steps and packages needed may vary on your machine.
1. Install nessesary Packages
   ```
   sudo apt install build-essential
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   exec bash
   ```

2. You can continue with [Setting up the Toolchain](#setting-up-the-toolchain).

### Setting up the Toolchain
1. Install the Rust Toolchain for xtensa-targets with the following commands
   ```
   cargo install espup
   espup install
   ```
2. The Installer will return a path to a file which has to be sourced.
   On Linux you need to add this file to the `~/.bash-rc` and restart the shell.
   On Windows you can add it to your PowerShell Profile.
   The path to this file can be obtained by running `$profile` inside a PowerShell Terminal.

3. To flash your compiled binarys we reccomend `espflash`.
   We recommend the version 2 and above as it integrates well with the cargo-build-system.
   To install it run
   ```
   cargo install espflash@"2.0.0-rc.3"
   ```
4. Plug your Microcontroller into your computer, put it into boot-mode if nessesary and run
   ```
   cargo run --release
   ```
   This only works if the correct version of espflash is installed.

   To buld and flash manually run
   ```
   cargo build --release
   espflash flash target/xtensa-esp32s3-none-elf/release/rush-service
   ```

   Be carful to only use binaries built with the `--release` profile, as the debug binaries can get too big to flash and the performance overhead is amplified by the comparatively slow hardware.

Now you should be able to connect to the wifi access point created by the microcontroller.
