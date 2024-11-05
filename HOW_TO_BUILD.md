
## How to Build

To build **rusteron-archive**, you need to have the following installed:

- **Rust**: Make sure you have the Rust toolchain installed. You can install it from [rustup.rs](https://rustup.rs/).
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
- **Java 17+**: Java 17 or newer is required to build the Aeron archive C bindings. Ensure that Java is properly set up in your system environment.
- **CMake**: required to build the c bindings
```shell
sudo apt install linux-tools-$(uname -r) cmake uuid-dev -y
# or
sudo snap install cmake --classic
```
- **CLang**: required to build the c bindings
```shell
sudo apt install clang -y
```
- **Just** - optional, similar to cmake
```shell
cargo install just
```

### Installing Java 17

#### macOS
On macOS, you can install Java 17 using [Homebrew](https://brew.sh/):

```bash
brew install openjdk@17
```

Then, link Java 17 to make it available system-wide:

```bash
sudo ln -sfn $(brew --prefix openjdk@17)/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk-17.jdk
```

#### Linux
On most Linux distributions, you can install OpenJDK 17 with the package manager:

- **Ubuntu/Debian**:
  ```bash
  sudo apt update
  sudo apt install openjdk-17-jdk
  ```

- **Fedora**:
  ```bash
  sudo dnf install java-17-openjdk
  ```

- **Arch Linux**:
  ```bash
  sudo pacman -S jdk-openjdk
  ```

#### Windows
On Windows, you can download and install Java 17 from the [Adoptium website](https://adoptium.net/), selecting the "Temurin" distribution and version 17. During installation, make sure to check the option to set up the Java environment variables.

### Building the Project

Once Rust and Java are installed, you can build the project using Cargo:

```bash
cargo build --release
```

If you are using a `just`, you can also run build commands conveniently with predefined tasks:

```bash
just build   # Builds the project
just test    # Runs the test suite
```

Make sure all dependencies are set up correctly to avoid issues with the Aeron C bindings during the build.

### Troubleshooting

* Complains no files in aeron directory
  * run 
```shell
git submodule update --init --recursive
```