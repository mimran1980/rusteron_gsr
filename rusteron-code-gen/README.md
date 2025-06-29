# rusteron-code-gen

**rusteron-code-gen** is a module in the **rusteron** project that is responsible for generating Rust code by analyzing the Aeron C bindings. It plays a crucial role in automating the generation of the Rust wrappers that make it possible for developers to interact with the Aeron APIs in an idiomatic Rust way.

## Overview

The **rusteron-code-gen** module is an internal tool used to generate Rust code based on Aeron's C bindings. Aeron follows certain patterns in its API, and **rusteron-code-gen** takes advantage of these predictable structures to automate much of the code generation process. By automating the conversion of C bindings to Rust, **rusteron-code-gen** helps reduce the time and effort required for maintaining and updating the Rust interface as the Aeron API evolves.

This module is used internally within the **rusteron** project and is not intended for direct use by developers integrating Aeron functionalities into their projects.

## Features

- **Automated Code Generation**: Generates Rust wrappers for the Aeron C bindings.
- **Pattern-Based Conversion**: Leverages Aeron's consistent API patterns to simplify Rust code generation.

## Project Status

**rusteron-code-gen** is in active use for generating and maintaining the Rust wrappers for other **rusteron** modules, such as **rusteron-client** and **rusteron-archive**. As the primary focus is on the client and archive modules, **rusteron-code-gen** does not have its own extensive testing beyond ensuring the generated code integrates smoothly into the rest of the **rusteron** project.

## Installation

**rusteron-code-gen** is used internally and is not intended to be added directly to other projects.

## Safety Considerations

Since **rusteron-code-gen** operates on C bindings, the generated code can include `unsafe` blocks to ensure compatibility with the low-level Aeron API. The automated process requires careful review and occasional manual intervention to ensure safety and correctness.

## Contributing

Contributions are more than welcome! Please see our [contributing guidelines](https://github.com/gsrxyz/rusteron/blob/main/CONTRIBUTING.md) for more information on how to get involved. Improvements to **rusteron-code-gen** could significantly enhance the automation and reliability of the Rust code generation process.

## License

This project is dual-licensed under either the [MIT License](https://opensource.org/licenses/MIT) or [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). You may choose which one to use.

## Links

- [GitHub Repository](https://github.com/gsrxyz/rusteron)

Feel free to reach out with any questions or suggestions via GitHub Issues!