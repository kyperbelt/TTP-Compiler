# TTP Compiler

![](https://github.com/kyperbelt/TTP_Compiler/actions/workflows/rust.yml/badge.svg)

### ⚠️DISCLAIMER⚠️ 
> TTPC is still under development and some features may not function as intended. If you notice something is not working correctly please open an Issue so that we can resolve it asap.

A compiler/assembler for files written in TTPASM to work with Tak's Toy Processor. 

## Contents
* [Build](#how-to-build)
* [Getting Started](#getting-started)
* [Tips](#tips)
* [Todo](#todo)
* [Credits](#credits)

## How to Build
To build TTP_Compiler you will need the latest version of the [Rust](https://www.rust-lang.org/tools/install) tool-chain.
Once you have that installed and setup on your system you should clone this repository to a folder on your local machine.

You can [download the repository](https://github.com/kyperbelt/TTP_Compiler/archive/refs/heads/main.zip) and unzip it. 
or use git to clone it to your machine with the following command:
```
git clone https://github.com/kyperbelt/TTP_Compiler.git
```
One you have that cloned you can run the following command from the project folder:
```
cargo build --release
```
The binary file should be located in the `<project>/target/release/` folder.

## Getting Started
[*NOTE*]:`if you downloaded one of the release binaries, you must rename it or use the full name to call it`

To get started using ttpc to convert your beautiful ttpasm into logisim loadable machine code you simply run ttpc as follows:
### Windows
```
ttpc -c <yourfile>
```
### Linux
```
./ttpc -c <yourfile>
```
This will assemble your file and output it into the same folder. It will use the same name as the input file sans extension.

If you would like to specify your own output file-name/file-path you can do the following:
###
```
./ttpc -c <inputfile.ttpasm> -o <outputfile>
```
The order of the commands does not matter, but some commands are dependant on others. 
For a full list of commands run `ttpc` with the `--help` or `-h` commands.

## Tips
* You can add the `target/release/` folder to your environment Path so you can can call `ttpc` from anywhere!

## Todo:
An assortment of tasks that still need implementing.
- [x] Add virtual machine to help with analysis.
- [ ] Test suite that ensures feature parity between online assembler.
- [ ] Add preprocessing for function returns using register D and stack data structure.
- [ ] Output to binary.(why? because it would be cool!)

## Credits:
* Tak Auyeung | auyeunt@arc.losrios.edu - for creating tpp and ttpasm, without which this project wouldn't exist.  
* [Nes370](https://github.com/Nes370) - for helping with suggestions and debugging. 
