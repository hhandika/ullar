# ULLAR

![ci](https://github.com/hhandika/ullar/workflows/tests/badge.svg)

ULLAR, named after _ular_, which means snakes in Indonesian/Malay, is an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. What sets ULLAR apart from other pipelines? Let's delve into its unique features that address the current limitations in the field.

## Performance and Scalability

ULLAR stands out with its exceptional Performance and Scalability. Unlike most pipelines written in scripting languages (e.g., Python or R) or memory-inefficient languages (e.g., Java) and struggle with performance optimization, ULLAR is designed to be ultrafast and scalable, minimizing the pipeline's overhead.

## Accessibility and Reproducibility

1. ULLAR's goal is to be easy to install and easy to use. We want to minimize dependencies, avoid containerization (e.g., using Docker or Singularity), and make the pipeline as self-contained as possible. Whenever possible, we will interop with available tools using the Foreign-Function Interface (FFI) and a static-linked binary to minimize runtime dependencies. We will also simplify command arguments and accompany the pipeline with comprehensive documentation. This way, non-tech-savvy users or those without technical support can easily install the pipeline and efficiently conduct phylogenomic analyses.

2. ULLAR is committed to supporting Windows for part of the pipeline, recognizing the importance of cross-platform compatibility in the field of bioinformatics. By prioritizing cross-platform tools available in Windows, we aim to improve reproducibility and accessibility, ensuring that all users, regardless of their operating system, can benefit from ULLAR's capabilities.

3. ULLAR will be reproducible through log output and runner scripts, and each input and output file will be accompanied by SHA256 hash values for data integrity checks. It will also provide a checkpoint to resume the pipeline. There will be no more rerunning the whole pipeline when it fails in the middle or manual hacks to resume it.

## Getting Started

ULLAR is currently under development. We are working on the pipeline's core components. You can try the pipeline by following the installation guide below. This guideline assume familiarity with the command line interface and basic bioinformatics tools.

### Installation

If you have Rust installed, you can install ULLAR using Cargo:

```bash
cargo install ullar
```

To install ULLAR, you can download the latest release from the [release page](https://github.com/hhandika/ullar/releases/latest). Available binaries:

| OS      |  Download |
|---------|--------|
| Linux   | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-x86_64.tar.gz) or [Many Linux Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-musl-x86_64.tar.gz) |
| Windows | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Windows-x86_64.zip) |
| MacOS   | [Intel](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-x86_64.tar.gz) or [M series](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-arm64.tar.gz) |

Install ULLAR like installing any single executable binary. For example, in Linux:

```bash
tar -xvf ullar-Linux-x86_64.tar.gz
```

Copy to your bin directory such as `/usr/local/bin`:

```bash
sudo cp ullar /usr/local/bin
```

or our home directory that is in the PATH if you don't have root access:

```bash
cp ullar ~/bin
```

SEGUL provide a detailed installation guide on installing a single executable binary in different operating systems. You can find the guide [here](https://www.segul.app/docs/installation/install_binary)

### Dependencies

- [SPAdes](http://cab.spbu.ru/software/spades/)
- [Fastp](https://github.com/OpenGene/fastp)

You can check if you have the dependencies installed by running the following commands:

```bash
ullar deps check
```

If you don't have the dependencies installed, you can install by following the instructions on the links provided above.

### Usage

Check ULLAR installation:

```bash
ullar --version
```

#### Generate a configuration file

```bash
ullar new /raw_read_dir
```

To check the configuration file:

```bash
cat configs/raw_read.yaml
```

The default argument assume your file has simple names:

```text
- sample1_R1.fastq.gz
- sample1_R2.fastq.gz
```

For more descriptive names, you can use the `--sample-name descriptive` argument:

```bash
ullar new /raw_read_dir --sample-name descriptive
```

Example of descriptive names:

```text
- sample1_Species1_R1.fastq.gz
- sample1_Species1_R2.fastq.gz
- genus1_species1_locality_R1.fastq.gz
- genus1_species1_locality_R2.fastq.gz
- genus1_species2_locality_R1.fastq.gz
- genus1_species2_locality_R2.fastq.gz
```

You can also supply your own regular expression to extract the sample name:

```bash
ullar new /raw_read_dir --re-sample='([a-zA-Z0-9]+)_R1.fastq.gz'
```

#### Cleaning raw reads

By default, ULLAR is using dry-run mode.

```bash
ullar clean -c configs/raw_read.yaml
```

To run the cleaning process:

```bash
ullar clean -c configs/raw_read.yaml --process
```

It will first check the config file and the hash values match the raw reads. For a fresh run, you can skip the hash check:

```bash
ullar clean -c configs/raw_read.yaml --process --skip-config-check
```

## De Novo Assembly

ULLAR uses SPAdes for de novo assembly. To run the assembly:

```bash
ullar assemble -c configs/cleaned_read.yaml --process
```
