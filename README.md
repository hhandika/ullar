# ULLAR <img src="https://raw.githubusercontent.com/hhandika/ullar/main/assets/icons/ullar-dark.png" alt="ullar logo" align="right" width="150"/>

![ci](https://github.com/hhandika/ullar/workflows/tests/badge.svg)
[![LoC](https://tokei.rs/b1/github/hhandika/ullar?category=code)](https://github.com/XAMPPRocky/tokei)

ULLAR stands for an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. Our goal with ULLAR is to develop a lightweight and scalable pipeline that requires a minimal learning curve and bioinformatic knowledge. In addition to Linux and macOS, the typical supported operating systems for bioinformatics, ULLAR will, whenever possible, run natively on Windows.

## Motivation

Wrangling the 'snakes' of phylogenomic pipelines can be a frustrating endeavor. Common issues with currently available genomic pipelines:
1. Inefficient and difficult to debug due to additional layers of abstraction and dependencies (e.g., SnakeMake, NextFlow, Python runtime, and dozens of other runtime dependencies).
2. Requires users to prepare config files, which can be tedious for those with limited knowledge of Shell scripting and end up doing it manually instead.
3. Sequence samples from non-model organisms often are not of ideal quality. An extra quality check and manual inspection at each step of the workflow is usually required to ensure optimal, accurate results.
4. Some HPC Clusters offer users limited privileges. Workflows that automatically submit >1,000 jobs are often not allowed in those systems.
5. Forced users to install all the dependencies. For instance, a user installing and reinstalling a pipeline, or having different pipelines for different genomic analyses, could end up with multiple SPAdes installed on the same computer.

We develop ULLAR to solve those problems. ULLAR is our baby step toward our long-term goals to ensure phylogenomic analyses are efficient and accessible to as many evolutionary biologists as possible, regardless of their technical skills and support. 

## Development Status

ULLAR is currently under development. We have completed the core components of the pipeline. However, you should expect command changes in the future release as we continue to refine the tool. If you use ULLAR in a publication, we recommend you specify the exact version of the app. For manual compilation, we also recommend that you state the commit hash number. For example, `ULLAR v0.3.0 (commit: f18ac98)`.

## Try ULLAR

### Installation

The ULLAR pipeline itself is a single-executable binary. Currently, you can install it as a binary or from source code. The stable release is also planned to be available on Bioconda. You can download the latest release from the [release page](https://github.com/hhandika/ullar/releases/latest). Available binaries for supported operating systems:

| OS      | Download                                                                                                                                                                                                                             |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Linux   | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-x86_64.tar.gz) or [Many Linux Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Linux-musl-x86_64.tar.gz) |
| Windows | [Intel/AMD 64-bit](https://github.com/hhandika/ullar/releases/latest/download/ullar-Windows-x86_64.zip)                                                                                                                              |
| MacOS   | [Intel](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-x86_64.tar.gz) or [M series](https://github.com/hhandika/ullar/releases/latest/download/ullar-macOS-arm64.tar.gz)                                     |

Install ULLAR like you would install any other single-executable binary. For example, in Linux:

```bash
tar -xvf ullar-Linux-x86_64.tar.gz
```

Copy to your bin directory, such as `/usr/local/bin`:

```bash
sudo cp ullar /usr/local/bin
```

Or our home directory that is in the PATH if you don't have root access:

```bash
cp ullar ~/bin
```

Check ULLAR installation:

```bash
ullar --version
```

You can also install ULLAR from source code using the Rust package manager, `cargo`. Follow the Rust installation guide [here](https://www.rust-lang.org/tools/install). After installing Rust, you can install using the following command in your terminal application:

```bash
cargo install --git https://github.com/hhandika/ullar.git
```

If you need more detailed guidelines, SEGUL provides comprehensive instructions for installing Rust-based software [here](https://www.segul.app/docs/installation/install_source).

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

> NOTE: Summary statistics and other data cleaning features are under development, but you can install SEGUL separately.
> Check out SEGUL documentation [here](https://www.segul.app/)

You can check if you have the dependencies installed by running the following commands:

```bash
ullar deps check
```

By default, ULLAR will use available dependencies in your system. For missing dependencies, the current option is to manually install them by following the instructions from the dependency providers. We are working on adding functionality to install dependencies automatically.

### Quick Start

For a complete analysis, consider using [SEGUL](https://www.segul.app/) for data cleaning and summary statistics. Eventually, all essential features from SEGUL will be integrated into ULLAR, eliminating the need to install SEGUL separately.

```bash
# Step 1: Clean raw reads
ullar clean init -d /raw_read_dir --autorun

# Step 2: De novo assembly
ullar assemble init -d /cleaned_read_dir --autorun

# Step 3: Reference mapping
ullar map init -d /cleaned_read_dir --reference /path/to/reference.fasta --reference-type probes --autorun
# or if your reference is a locus
ullar map init -d /cleaned_read_dir --reference /path/to/reference.fasta --reference-type loci --autorun

# Step 4: Sequence alignment
ullar align init -d /path/to/unaligned_sequences_dir --autorun

# Step 5: Phylogenetic analysis
ullar tree init -d /path/to/aligned_sequences_dir --autorun
```

### Detailed Steps

#### Cleaning raw reads

A quick way to clean reads:

```bash
ullar clean init -d /raw_read_dir --autorun
```

By default ullar use descriptive name format to match the sample. It is equal to running ullar using this argument:

```bash
ullar clean init -d /raw_read_dir --sample-name descriptive
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

If your file naming is simple, you can use the `--sample-name simple` argument:

```bash
ullar clean init -d /raw_read_dir --sample-name simple
```

Example of simple names:

```text
- sample1_R1.fastq.gz
- sample1_R2.fastq.gz
```

You can also supply your own regular expression to extract the sample name:

```bash
ullar clean init -d /raw_read_dir --re-sample='([a-zA-Z0-9]+)_R1.fastq.gz'
```

To run the cleaning process:

```bash
ullar clean run -c configs/read_cleaning.toml
```

If you prefer to check the config file before running the cleaning process, you can init ullar without the `--autorun` argument:

```bash
ullar clean init -d /raw_read_dir
```

To run the cleaning process, you can use the `--skip-config-check` argument to skip the config check:

```bash
ullar clean run -c configs/read_cleaning.toml
```

#### De Novo Assembly

ULLAR uses SPAdes for de novo assembly. To run the assembly:

```bash
ullar assemble init -d /cleaned_read_dir --autorun
```

If you prefer to check the config file before running the assembly process, you can init ullar without the `--autorun` argument:

```bash
ullar assemble init -d /cleaned_read_dir
```

To run the assembly process, you can use the `--skip-config-check` argument to skip the config check:

```bash
ullar assemble run -c configs/de_novo_assembly.toml
```

#### Reference Mapping

ULLAR uses LASTZ for reference mapping. To run the mapping:

```bash
ullar map init -d /path/to/cleaned_read_dir --reference /path/to/reference.fasta --reference-type probes --autorun
```

If your reference is a locus, you can use the `--reference-type loci` argument:

```bash
ullar map init -d /path/to/cleaned_read_dir --reference /path/to/reference.fasta --reference-type loci --autorun
```

For the `probes` type, ULLAR will pull an entire contig that matches the probe. The output will be in Lastz `general` format and sequence files in FASTA format.

For the `loci` type, ULLAR will only pull the part of the contig that matches the reference. The output will be in Multi Alignment Format (MAF) and FASTA format.

#### Sequence Alignment

ULLAR uses MAFFT for sequence alignment. To run the sequence alignment:

```bash
ullar align init -d /path/to/unaligned_sequences_dir --autorun
```

If you prefer to check the config file before running the alignment process, you can init ullar without the `--autorun` argument:

```bash
ullar align init -d /path/to/unaligned_sequences_dir
```

To run the alignment process, you can use the `--skip-config-check` argument to skip the config check:

```bash
ullar align align -c configs/sequence_alignment.toml
```

#### Phylogenetic Analysis

ULLAR uses IQ-TREE for phylogenetic analysis. To run the phylogenetic analysis:

```bash
ullar tree init -d /path/to/aligned_sequences_dir --autorun
```

If you prefer to check the config file before running the phylogenetic analysis process, you can init ullar without the `--autorun` argument:

```bash
ullar tree init -d /path/to/aligned_sequences_dir
```

To run the phylogenetic analysis process, you can use the `--skip-config-check` argument to skip the config check:

```bash
ullar tree run -c configs/phylogenetic_analysis.toml
```

To specify the tree inference method, you can use the `--specify-analyses` argument:

Options are:

- `ml-species` for maximum likelihood species tree inference
- `ml-genes` for maximum likelihood gene tree inference
- `gscf` for gene site concordance factor
- `msc` for multi-species coalescent tree inference

For example:

```bash
ullar tree init -d /path/to/aligned_sequences_dir --specify-analyses ml-species ml-genes --autorun
```

You can also specify the multi-species coalescent tree inference method using `--specify-msc-methods` argument:

Options are:

- `astral` for ASTRAL 4 methods.
- `astral-pro` for ASTRAL Pro methods.
- `wastral` for Weighted ASTRAL methods.

MSC methods require `--specify-analyses ml-genes msc` argument.

For example:

```bash
ullar tree init -d /path/to/aligned_sequences_dir --specify-analyses ml-genes msc --specify-msc-methods astral-pro --autorun
```
