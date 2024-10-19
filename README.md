# ULLAR

![ci](https://github.com/hhandika/ullar/workflows/tests/badge.svg)

ULLAR, named after _ular_, which means snakes in Indonesian/Malay, stands for an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. Our goal with ULLAR is to develop a pipeline that is lightweight and requires minimal learning curve. We wants a scalable pipeline that can be used by evolutionary biologists with limited bioinformatics background and technical supports. Whenever possible, ULLAR feature will run native on Windows as well as Linux and MacOS.

## Getting Started

ULLAR is currently under development. We are working on the pipeline's core components. You can try the pipeline by following the installation guide below. This guideline assume familiarity with the command line interface and basic bioinformatics tools.

### Installation

Currently, ULLAR installation requires Rust. Follow Rust installation guide [here](https://www.rust-lang.org/tools/install). After installing Rust, you can install ULLAR using cargo:

```bash
cargo install --git https://github.com/hhandika/ullar.git
```

<!-- Another option is to install ULLAR pre-compiled binary. You can download the latest release from the [release page](https://github.com/hhandika/ullar/releases/latest). Available binaries:

| OS      | Download                                                                                                                                                                                                                             |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Linux   | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-x86_64.tar.gz) or [Many Linux Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-musl-x86_64.tar.gz) |
| Windows | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Windows-x86_64.zip)                                                                                                                              |
| MacOS   | [Intel](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-x86_64.tar.gz) or [M series](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-arm64.tar.gz)                                     |

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
``` -->

SEGUL provide a detailed installation guide on installing Rust based software [here](https://www.segul.app/docs/installation/install_source)

### Features & Dependencies

| Feature                             | Dependencies                                       |
| ----------------------------------- | -------------------------------------------------- |
| Raw read cleaning                   | [Fastp](https://github.com/OpenGene/fastp)         |
| De novo assembly                    | [SPAdes](http://cab.spbu.ru/software/spades/)      |
| Reference mapping                   | [LASTZ](https://github.com/lastz/lastz)            |
| Sequence alignment                  | [MAFFT](https://mafft.cbrc.jp/alignment/software/) |
| ML phylogeny (in development)       | [IQ-TREE](http://www.iqtree.org/)                  |
| MSC phylogeny (in development)      | [ASTER](https://github.com/chaoszhang/ASTER)       |
| Data cleaning                       | [SEGUL](https://www.segul.app/)                    |
| Summary statistics (in development) | [SEGUL](https://www.segul.app/)                    |

> NOTE: Summary statistics and other data cleaning feature is under development, but you can install SEGUL separately.
> Check out SEGUL documentation [here](https://www.segul.app/)

You can check if you have the dependencies installed by running the following commands:

```bash
ullar deps check
```

If you don't have the dependencies installed, you can install by following the instructions on the links provided above.

Check ULLAR installation:

```bash
ullar --version
```

#### Generate a config file

```bash
ullar clean init /raw_read_dir
```

To check the config file:

```bash
cat configs/clean_read.yaml
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
ullar clean -c configs/read_cleaning.yaml
```

To run the cleaning process:

```bash
ullar clean -c configs/read_cleaning.yaml --process
```

It will first check the config file and the hash values match the raw reads. For a fresh run, you can skip the hash check:

```bash
ullar clean -c configs/read_cleaning.yaml --process --skip-config-check
```

#### De Novo Assembly

ULLAR uses SPAdes for de novo assembly. To run the assembly:

```bash
ullar assemble -c configs/denovo_assembly.yaml --process
```

#### Reference Mapping

ULLAR uses LASTZ for reference mapping. To run the reference mapping:

```bash
ullar map -c configs/reference_mapping.yaml --process
```
