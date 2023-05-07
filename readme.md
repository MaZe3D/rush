# rush - remote shell for microcontrollers
rush is a simple shell to interact with various peripherals on a microcontroller (currently the ESP32-S3), like GPIO, UART (Serial) and addressable LEDs

## Build Envoirement
Setting up the build-envoirement is possible for Windows and Linux and we provide a Docker-Image.


### Prerequisites on Windows
On Windows (version 10 and 11 respectevly) you need to follow these steps:
1. You need a supoorted Linker on your system. If you have a Linker installed, you can ignore this step. For windows we reccomend the [Microsoft Visual Studio Build Tools](), which can be obtained via winget:
   ```
   winget install Microsoft.VisualStudio.2022.BuildTools --interactive
   ```

2. Inside the Visual Studion installer select the `C++ Development Tools`

3. Install the rust-toolchain installer [rustup](https://rustup.rs/). This can be done by downloading it from the official site or via winget via the command:
    ```
    winget install Rustlang.Rustup
    ```

4. You can continue with [Setting up the Toolchain](#setting-up-the-toolchain)

### Prerequisites on Linux
The following steps are made for Ubuntu and other Debian-Based Distributoions. The exact steps and packackages needed may vary on your machine.
1. Install nessesary Packages
    ```
    sudo apt install build-essential
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    exec bash
    ```

2. You can continue with [Setting up the Toolchain](#setting-up-the-toolchain)

### Setting up the Toolchain
1. Install the Rust Toolchain for xtensa-targets with the following commands
    ```
    cargo install espup
    espup install
    ```
2. The Installer will return you a path to a file. On Linux you need to add this file to the `~/.bash-rc` and run `exec bash`. On Windows you can add it to your PowerShell Profile. The path to this file can be obtained by running `$profile` inside a PowerShell Terminal.

3. To flash your compiled binarys we reccomend `espflash`. We reccomend the version 2 and above as it integrates well with the cargo-build-system. To install it run
    ```
    cargo install espflash@"2.0.0-rc.3"
    ```
4. Plug your Microcontroller into your computer, put it into boot-mode if nessesary and run 
    ```
    cargo run --release
    ```

Now you should see the serial output of your microcontroller.
