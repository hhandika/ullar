# ULLAR <img src="https://raw.githubusercontent.com/hhandika/ullar/main/assets/icons/ullar-dark.png" alt="ullar logo" align="right" width="150"/>

![ci](https://github.com/hhandika/ullar/workflows/tests/badge.svg)

ULLAR, named after _ular_, which means snakes in the Indonesian language, stands for an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. Our goal with ULLAR is to develop a lightweight and scalable pipeline that requires a minimal learning curve. In addition to Linux and MacOS, the typical supported operating systems for bioinformatics, whenever possible, ULLAR will run natively on Windows.

## Motivation

Building a pipeline with general-purpose workflow managers such as Snakemake or Nextflow introduces an additional layer of abstraction to an already complex process. This added complexity can make pipelines more difficult to debug, modify, and maintain. To overcome these limitations, we need a pipeline developed from the ground up using a programming language designed for speed, safety, and concurrency, without requiring extra runtime dependencies. ULLAR is carefully optimized to run efficiently, even on limited computational resources and with minimal technical support. Our primary goal is to enable researchers working on non-model organisms to perform a complete phylogenomic analysis with a single command, while still offering the flexibility to modify and customize individual steps as needed.

## Development Status

ULLAR is currently under development. We are working on the pipeline's core components. You should expect command changes in the future release. If you use ULLAR in publication, we recommend stating the exact version of the app. For manual compilation, we recommend to also state the commit hash number. For example, `ULLAR v0.3.0 (commit: f18ac98)`.

## Try ULLAR

You can try the pipeline by following the installation guide below. This guideline assume familiarity of using command line app and basic bioinformatics tools.

### Installation

Currently, ULLAR installation requires Rust. Follow Rust installation guide [here](https://www.rust-lang.org/tools/install). After installing Rust, you can install ULLAR using cargo:

```bash
cargo install --git https://github.com/hhandika/ullar.git
```

Another option is to install ULLAR pre-compiled binary. You can download the latest release from the [release page](https://github.com/hhandika/ullar/releases/latest). Available binaries:

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
```

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
ullar clean run -c configs/read_cleaning.yaml
```

If you prefer to check the config file before running the cleaning process, you can init ullar without the `--autorun` argument:

```bash
ullar clean init -d /raw_read_dir
```

To run the cleaning process, you can use the `--skip-config-check` argument to skip the config check:

```bash
ullar clean run -c configs/read_cleaning.yaml
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
ullar assemble run -c configs/de_novo_assembly.yaml
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

For the `probes` type, ULLAR will pull an entire contig that matches the probe. The output will in lastz `general` format and sequence files in FASTA format.

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
ullar align run -c configs/sequence_alignment.yaml
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
ullar tree run -c configs/phylogenetic_analysis.yaml
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

MSC methods requires `--specify-analyses ml-genes msc` argument.

For example:

```bash
ullar tree init -d /path/to/aligned_sequences_dir --specify-analyses ml-genes msc --specify-msc-methods astral-pro --autorun
```
