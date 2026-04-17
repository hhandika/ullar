# ULLAR <img src="https://raw.githubusercontent.com/hhandika/ullar/main/assets/icons/ullar-dark.png" alt="ullar logo" align="right" width="150"/>

![ci](https://github.com/hhandika/ullar/workflows/tests/badge.svg)
![GitHub top language](https://img.shields.io/github/languages/top/hhandika/ullar)
![GitHub last commit](https://img.shields.io/github/last-commit/hhandika/ullar?color=yellow)

ULLAR is a lightweight, efficient, and scalable pipeline developed to minimize learning curve and required bioinformatic knowledge for phylogenomic and population genetic data analyses. We defined efficiency as efficient in both using computational resources and user time.

For starter, ULLAR is a single executable. This approach avoids potential runtime dependency conflicts with the genomic applications it relies on. We eliminate the need to prepare configuration files and scripts. The app also automatically infers optimal resource allocation based on the data and available resources, but provides flexibility for users to override the default settings. See the [Motivation](#motivation) section for more details of our design goals.

In addition to Linux and macOS, the most commonly supported operating systems for bioinformatics, ULLAR runs natively on Windows whenever possible.

## Name Origin

ULLAR stands for an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. The name "ULLAR" (pronounced _oo-lar_) is inspired by the Indonesian/Malay word for snake: _ular_.

## Motivation

ULLAR is initially developed to support teaching phylogenomics in two-day workshops. As the development progressed, our goals and motivations expanded: how can we design a simple-to-use pipeline that is also efficient and scalable? We identified several issues in existing pipelines that we aim to address:

1. Inefficient and difficult to debug due to additional layers of abstraction and runtime dependencies, such as [SnakeMake](https://snakemake.readthedocs.io/en/stable/)/[NextFlow](https://www.nextflow.io/), Python/Java runtime, and dozens of additional runtime dependencies.
2. Requires users to prepare config files, which can be tedious for those with limited Shell-scripting experience and can lead to error-prone manual editing.
3. Sequence data from non-model organisms are often not ideal. An extra quality check and manual inspection at each step of the workflow is usually required to ensure optimal, accurate results. So, fully automatic pipeline often is not the right solution. How can we design a pipeline that make it easy for users to inspect intermediate results and tweak parameters at each step of the workflow if necessary?
4. Some HPC Clusters offer users limited privileges. Designing pipelines that does not require root access and can be installed in user space is crucial for accessibility.
5. Forced users to install all the dependencies. For instance, a user having different pipelines for different genomic analyses, could end up with multiple [SPAdes](http://cab.spbu.ru/software/spades/) installed on the same computer. How can we avoid duplicate installations of the same software and potential runtime dependency conflicts?
6. Native Windows support is largely absent from existing genomic pipelines, even though several key tools run on Windows (e.g., [IQ-TREE](https://iqtree.github.io/) for phylogenetic inference and [SEGUL](https://www.segul.app/) for data cleaning, summarization, and wrangling). How can we provide native support for Windows users for those parts of the workflow?
7. Concurrency and parallization are easy to get wrong. Some pipelines require users to specify the number of threads for each step manually. However, some steps are I/O bound, while others are CPU bound. How can we optimize resource allocation automatically based on the data, type of analyses, and available resources?

ULLAR is our baby step toward our long-term goals to ensure phylogenomic analyses are efficient and accessible to as many evolutionary biologists as possible, regardless of their technical skills and support.

## Development Status

ULLAR is currently under development. We have completed the core components of the pipeline. However, you should expect command changes in the future release as we continue to refine the tool. If you use ULLAR in a publication, we recommend you specify the exact version of the app. For manual compilation, we also recommend that you state the commit hash number. For example, `ULLAR v0.3.0 (commit: f18ac98)`. See the [Feature & Dependencies](#features--dependencies) section to see the current project progress and the dependencies that are currently supported.

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

#### Install from Source Code

You can also install ULLAR from source code. You will need `git` and rust compiler. Follow [the Rust installation guide](https://www.rust-lang.org/tools/install). After installing Rust, you can install it using the following command in your terminal application:

##### Clone the repository

```bash
git clone https://github.com/hhandika/ullar.git

# or using GitHub CLI
gh repo clone hhandika/ullar
```

```bash
cd ullar
```

##### Build and install ULLAR

ULLAR code is modular and organized into separate packages for different features. To install the main ULLAR package, clone the repository to your local machine and run the following command in your terminal application:

```bash
cd crates/ullar
cargo install --path .
```

To install other packages within ULLAR, such as `ullar-bwa`, you can run:

```bash
cd crates/ullar-bwa
cargo install --path .
```

If you need more detailed guidelines, SEGUL provides comprehensive instructions for installing Rust-based software in [the installation guide](https://www.segul.app/docs/installation/install_source).

### Features & Dependencies

#### Phylogenomics

| Feature            | Dependencies                                       | Status |
| ------------------ | -------------------------------------------------- | ------ |
| Raw read cleaning  | [Fastp](https://github.com/OpenGene/fastp)         | ☑️     |
| De novo assembly   | [SPAdes](http://cab.spbu.ru/software/spades/)      | ☑️     |
| Reference mapping  | [LASTZ](https://github.com/lastz/lastz)            | ☑️     |
| Sequence alignment | [MAFFT](https://mafft.cbrc.jp/alignment/software/) | ☑️     |
| ML phylogeny       | [IQ-TREE](http://www.iqtree.org/)                  | ☑️     |
| MSC phylogeny      | [ASTER](https://github.com/chaoszhang/ASTER)       | ☑️     |
| Data cleaning      | [SEGUL](https://www.segul.app/)                    | ⏱️     |
| Summary statistics | [SEGUL](https://www.segul.app/)                    | ⏱️     |

> NOTE: Summary statistics and other data cleaning features are under development, but you can install SEGUL separately.
> Check out [SEGUL documentation](https://www.segul.app/)

### Population Genomics

| Feature           | Dependencies                                                                                         | Status |
| ----------------- | ---------------------------------------------------------------------------------------------------- | ------ |
| Raw read cleaning | [Fastp](https://github.com/OpenGene/fastp)                                                           | ☑️     |
| Read mapping      | [BWA](https://github.com/lh3/bwa)                                                                    | ☑️     |
| Mark duplicates   | [Sambamba](https://github.com/biod/sambamba)                                                         | ☑️     |
| Variant calling   | [GATK](https://gatk.broadinstitute.org/hc/en-us) or [BCFtools](https://github.com/samtools/bcftools) | ☑️     |
| DB import         | [GATK](https://gatk.broadinstitute.org/hc/en-us)                                                     | ⏱️     |
| Joint genotyping  | [GATK](https://gatk.broadinstitute.org/hc/en-us)                                                     | ⏱️     |
| Variant filtering | [GATK](https://gatk.broadinstitute.org/hc/en-us)                                                     | ⏱️     |

You can check if you have the dependencies installed by running the following commands:

```bash
ullar deps check
```

By default, ULLAR will use available dependencies in your system. For missing dependencies, the current option is to manually install them by following the instructions from the dependency providers. We are working on adding functionality to install dependencies automatically.

### Quick Start

For a complete analysis, consider using [SEGUL](https://www.segul.app/) for alignment cleaning and summary statistics. Eventually, all essential features from SEGUL will be integrated into ULLAR, eliminating the need to install SEGUL separately.

#### Phylogenomic Workflow

```bash
# Step 1: Clean raw reads
ullar clean init -d /raw_read_dir --autorun

# Step 2: De novo assembly
ullar assemble init -d /cleaned_read_dir --autorun

# Step 3: Reference mapping
# For a probe-based reference, such as Ultra-Conserved Elements (UCEs) phylogenomics
ullar map init -d /cleaned_read_dir --reference /path/to/reference.fasta --reference-type probes --autorun
# or if your reference is a locus, such as mitochondrial genome or nuclear genes
ullar map init -d /cleaned_read_dir --reference /path/to/reference.fasta --reference-type loci --autorun

# Step 4: Sequence alignment
ullar align init -d /path/to/unaligned_sequences_dir --autorun

# Step 5: Alignment cleaning and data processing (optional, recommended)
# For now, use SEGUL for alignment cleaning: https://www.segul.app/
# Step 5.1 Checking alignment quality
segul align summary --dir path/to/aligned_sequences_dir
# Step 5.2 Filter by parsimony informative sites of 1
segul align filter -d mafft-nexus-edge-trimmed-segul-clean/  --pinf 1
# Step 5.3 Filter alignments containing 80 percent of samples
segul align filter --dir mafft-nexus-edge-trimmed-segul-clean/  \
--percent .8 \
-o output/path

# Step 5: Phylogenetic analysis
ullar tree init -d /path/to/aligned_sequences_dir --autorun
```

#### Population Genomic Workflow

This workflow is under development. Currently, it requires installation of separate packages within ULLAR, such as `ullar-bwa` for read mapping. The final version will integrate all components within ULLAR itself, making it only a single executable binary. It will also offers separate workflow installation for those who only need specific workflows.

```bash
# Step 1: Clean raw reads
# Skip if you have cleaned reads already
ullar clean init -d /raw_read_dir --autorun


# Step 2: Map reads to reference genome
# In development, available through ullar-bwa crate
# Support BWA and BWA-MEM2 aligners
# Recursive flags look for reads in subdirectories, required if cleaned reads are processed using ullar clean
# Step 2.1: Index the reference genome
# Require ullar-bwa and BWA installation
# This will index the reference genome,
# create fai index using samtools, and
# create sequence dictionary using GATK
ullar-bwa index /path/to/reference_genome.fasta
# Step 2.2: Map reads to the reference genome
# ullar-bwa auto infers read group based on the fastq headers
ullar-bwa batch -d /cleaned_read_dir --reference /path/to/reference_genome.fasta --threads 8 --recursive

# Step 3: Mark duplicates
# Require sambamba installation: https://github.com/biod/sambamba
ullar-sambamba markdup -d /mapped_reads_dir --recursive --autorun

# Step 4: Variant calling using BCFtools
# Require ullar-bcftools and BCFtools installation: https://github.com/samtools/bcftools
ullar-bcftools call -d /marked_duplicates_bam_dir --reference /path/to/reference_genome.fasta --ploidy 2
```

GATK variant calling is under development. Only the variant calling step is completed.

```bash
# Step 4: Variant calling
# Require ullar-gatk and GATK installation
ullar-gatk variant -d /marked_duplicates_bam_dir --reference /path/to/reference

# Step 5: DB import (under development)
ullar-gatk db -d /variant_calling_dir --reference /path/to/reference

# Step 6: Joint genotyping (under development)
ullar-gatk joint -d /db_import_dir --reference /path/to/reference

# Step 7: Variant filtering (under development)
ullar-gatk filter -d /joint_genotyping_dir --reference /path/to/reference
```

### Detailed Usage

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

## Acknowledgements

We would like to thank the developers of the dependencies ULLAR relies on, including Fastp, SPAdes, LASTZ, MAFFT, IQ-TREE, ASTRAL, and SEGUL. We also want to thank the open-source community for their contributions to the development of libraries and tools that we use in ULLAR. The following people provided valuable feedback and suggestions during the development of ULLAR: Diego J. Elias, Austin S. Chipps, Giovani Hernández-Canchola, Spenser J. Babb-Biernacki, Sheila Rodríguez Machado, and Veronika Chalupová.Funding for this project was provided by the National Science Foundation ([DEB-1754393](https://www.nsf.gov/awardsearch/showAward?AWD_ID=1754393) and [DEB-2244754](https://www.nsf.gov/awardsearch/showAward?AWD_ID=2244754)) and the Alfred L. Gardner and Mark S. Hafner Mammalogy Fund. Several features of ULLAR are inspired by [Phyluce](https://phyluce.readthedocs.io/en/latest/), [FrogCap](https://github.com/chutter/FrogCap-Sequence-Capture), [snpArcher](https://snparcher.readthedocs.io/en/latest/), and [HybPiper](https://github.com/mossmatters/HybPiper).
