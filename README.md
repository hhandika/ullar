# ullar

ULLAR is an Ultrafast, scaLable, Accessible, and Reproducible pipeline for phylogenomics. Why do need another pipeline for phylogenomics? Our main reasons:

## Performance and Scalability

1. Most pipelines are written in **scripting languages** (e.g., Python, R) and are not optimized for performance. While for most pipelines, the heavy lifting is done by external tools (often are fast) the overhead of the pipeline itself can be significant, especially when some work is done within the pipeline. ULLAR is aimed to minimize this overhead.

2. Most pipelines are not optimized for **parallelism**. Whenever possible we want to take advantage of multi-core systems. We will do it safely to avoid data race.

## Accessibility and Reproducibility

1. ULLAR's goal is to be **easy to install** and **easy to use**. We want to minimize dependencies, avoid containerization (e.g., using Docker or Singularity), and make the pipeline as self-contained as possible. Whenever possible we will interact with available tools using Foreign-Function Interface (FFI), so the tool will be contained within the ULLAR binary. This way, non-tech-savvy users or those without technical support can easily install and conduct phylogenomic analyses.

2. ULLAR will **support Windows** for part of the pipeline. Most of day-to-day user are using Windows. We understand that many bioinformatic tools are not available for Windows, but we wanted to make sure that cross-platform tools that are available in Windows it will be supported by ULLAR. We believe it will improve reproducibility and accessibility. ULLAR approach will also prioritize tools that are available in Windows.

3. ULLAR will not only be **reproducible** through log output and a runner script, but will also provide check point to resume the pipeline. No more rerunning the whole pipeline when it fails in the middle or do manual hacks to resume the pipeline.
